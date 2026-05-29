use anyhow::Result;
use clap::{Parser,Subcommand};
use server::{build_app,AppState};
use std::{net::SocketAddr,sync::Arc};
use tracing::info;

#[derive(Parser)]
#[command(name="forgefabrik",about="AI-Agenten bauen und kämpfen in einer Voxel-Welt.",version)]
struct Cli{#[command(subcommand)]command:Cmd}

#[derive(Subcommand)]
enum Cmd{
    Serve{#[arg(short,long,default_value="8080")]port:u16,#[arg(short,long,default_value="42")]seed:u64},
    Spawn{#[arg(short,long)]name:String,#[arg(short,long,default_value="claude")]kind:String,#[arg(long,default_value="http://localhost:8080")]api:String},
    Status{#[arg(long,default_value="http://localhost:8080")]api:String},
    Watch{#[arg(long,default_value="http://localhost:8080")]api:String},
}

#[tokio::main]
async fn main()->Result<()>{
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("forgefabrik=info".parse()?))
        .with_target(false).init();
    match Cli::parse().command{
        Cmd::Serve{port,seed}=>serve(port,seed).await,
        Cmd::Spawn{name,kind,api}=>spawn(api,name,kind).await,
        Cmd::Status{api}=>status(api).await,
        Cmd::Watch{api}=>watch(api).await,
    }
}

async fn serve(port:u16,seed:u64)->Result<()>{
    info!(port,seed,"ForgeFabrik starten");
    let state=Arc::new(AppState::new(seed));
    let app=build_app(state);
    let addr=SocketAddr::from(([0,0,0,0],port));
    let listener=tokio::net::TcpListener::bind(addr).await?;
    info!(%addr,"bereit");
    axum::serve(listener,app).await?;
    Ok(())
}

async fn spawn(api:String,name:String,kind:String)->Result<()>{
    let body=serde_json::json!({"name":name,"kind":kind});
    let resp:serde_json::Value=reqwest::Client::new().post(format!("{api}/agents")).json(&body).send().await?.json().await?;
    println!("{}",serde_json::to_string_pretty(&resp)?);
    Ok(())
}

async fn status(api:String)->Result<()>{
    let w:serde_json::Value=reqwest::Client::new().get(format!("{api}/world")).send().await?.json().await?;
    let a:serde_json::Value=reqwest::Client::new().get(format!("{api}/agents")).send().await?.json().await?;
    println!("Welt:\n{}\nAgents:\n{}",serde_json::to_string_pretty(&w)?,serde_json::to_string_pretty(&a)?);
    Ok(())
}

async fn watch(api:String)->Result<()>{
    use crossterm::{event::{self,Event,KeyCode},execute,terminal::{disable_raw_mode,enable_raw_mode,EnterAlternateScreen,LeaveAlternateScreen}};
    use ratatui::{backend::CrosstermBackend,layout::{Constraint,Direction,Layout},style::{Color,Style},widgets::{Block,Borders,Paragraph},Terminal};
    enable_raw_mode()?;
    let mut stdout=std::io::stdout();
    execute!(stdout,EnterAlternateScreen)?;
    let mut terminal=Terminal::new(CrosstermBackend::new(stdout))?;
    let client=reqwest::Client::new();
    loop{
        let ws=client.get(format!("{api}/world")).send().await.and_then(|r|futures::executor::block_on(r.text())).unwrap_or_else(|e|e.to_string());
        let ag=client.get(format!("{api}/agents")).send().await.and_then(|r|futures::executor::block_on(r.text())).unwrap_or_else(|e|e.to_string());
        terminal.draw(|f|{
            let cs=Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(50),Constraint::Percentage(50)]).split(f.area());
            f.render_widget(Paragraph::new(ws.as_str()).block(Block::default().title(" Welt ").borders(Borders::ALL)).style(Style::default().fg(Color::Green)),cs[0]);
            f.render_widget(Paragraph::new(ag.as_str()).block(Block::default().title(" Agents ").borders(Borders::ALL)).style(Style::default().fg(Color::Cyan)),cs[1]);
        })?;
        if event::poll(std::time::Duration::from_millis(500))?{if let Event::Key(k)=event::read()?{if k.code==KeyCode::Char('q'){break;}}}
    }
    disable_raw_mode()?;execute!(terminal.backend_mut(),LeaveAlternateScreen)?;
    Ok(())
}
