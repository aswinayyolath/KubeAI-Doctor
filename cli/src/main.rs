use clap::{Arg, Command};
use colored::*;
use k8s_openapi::api::core::v1::{Node, Pod, Service};
use kube::{Api, Client, ResourceExt};
use kube::api::ListParams;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("KubeAI Doctor")
        .about("AI-powered Kubernetes troubleshooting tool")
        .arg(
            Arg::new("check")
                .short('c')
                .long("check")
                .value_name("RESOURCE")
                .help("Run a health check on a specific Kubernetes resource (e.g., nodes, pods, services, events)")
                .required(true),
        )
        .arg(
            Arg::new("namespace")
                .short('n')
                .long("namespace")
                .value_name("NAMESPACE")
                .help("Specify a Kubernetes namespace (default: all namespaces)"),
        )
        .get_matches();

    let namespace = matches.get_one::<String>("namespace").map(String::as_str);
    
    if let Some(resource) = matches.get_one::<String>("check") {
        match resource.as_str() {
            "nodes" => check_nodes().await?,
            "pods" => check_pods(namespace).await?,
            "services" => check_services(namespace).await?,
            "events" => check_events(namespace).await?,
            _ => eprintln!("{} Invalid resource. Use 'nodes', 'pods', 'services', or 'events'.", "[ERROR]".red()),
        }
    }
    Ok(())
}

async fn check_nodes() -> Result<()> {
    println!("{} Running health check on Kubernetes nodes...", "[INFO]".cyan());
    let client = Client::try_default().await?;
    let nodes: Api<Node> = Api::all(client);
    let node_list = nodes.list(&Default::default()).await?;

    let mut healthy = 0;
    let mut unhealthy = 0;

    for node in node_list.items {
        let name = node.name_any();
        let status = node.status.unwrap();
        let conditions = status.conditions.unwrap_or_default();
        
        if conditions.iter().any(|c| c.type_ == "Ready" && c.status == "True") {
            println!("✅ Node: {}", name.green());
            healthy += 1;
        } else {
            println!("❌ Node: {} (NotReady)", name.red());
            unhealthy += 1;
        }
    }
    println!("\n{} {} healthy, {} unhealthy", "[SUMMARY]".yellow(), healthy, unhealthy);
    Ok(())
}

async fn check_pods(namespace: Option<&str>) -> Result<()> {
    println!("{} Running health check on Kubernetes pods...", "[INFO]".cyan());
    let client = Client::try_default().await?;
    let pods: Api<Pod> = if let Some(ns) = namespace {
        Api::namespaced(client, ns)
    } else {
        Api::all(client)
    };
    let pod_list = pods.list(&ListParams::default()).await?;

    let mut healthy = 0;
    let mut unhealthy = 0;

    for pod in pod_list.items {
        let name = pod.name_any();
        let status = pod.status.unwrap();
        let phase = status.phase.unwrap_or_else(|| "Unknown".to_string());
        
        if phase == "Running" {
            println!("✅ Pod: {}", name.green());
            healthy += 1;
        } else {
            println!("❌ Pod: {} (Status: {})", name.red(), phase.red());
            unhealthy += 1;
        }
    }
    println!("\n{} {} healthy, {} unhealthy", "[SUMMARY]".yellow(), healthy, unhealthy);
    Ok(())
}

async fn check_services(namespace: Option<&str>) -> Result<()> {
    println!("{} Running health check on Kubernetes services...", "[INFO]".cyan());
    let client = Client::try_default().await?;
    let services: Api<Service> = if let Some(ns) = namespace {
        Api::namespaced(client, ns)
    } else {
        Api::all(client)
    };
    let service_list = services.list(&ListParams::default()).await?;

    for service in service_list.items {
        let name = service.name_any();
        println!("🔹 Service: {}", name.blue());
    }
    Ok(())
}

async fn check_events(namespace: Option<&str>) -> Result<()> {
    println!("{} Fetching recent Kubernetes events...", "[INFO]".cyan());
    let client = Client::try_default().await?;
    let events: Api<k8s_openapi::api::core::v1::Event> = if let Some(ns) = namespace {
        Api::namespaced(client, ns)
    } else {
        Api::all(client)
    };
    let event_list = events.list(&ListParams::default()).await?;

    for event in event_list.items {
        let name = event.name_any();
        let message = event.message.unwrap_or_else(|| "No message".to_string());
        println!("📢 Event: {} - {}", name.magenta(), message);
    }
    Ok(())
}