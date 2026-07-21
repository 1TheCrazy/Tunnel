use tunnel_core::structs::client::{ClientNode, ClientServer};
use tabled::{Table, Tabled, settings::{Alignment, Color, Modify, Style, object::{Columns, Cell}}};

use crate::write_line;

#[derive(Tabled)]
struct ClientNodeRow {
    #[tabled(rename = "ID")]
    id: String,

    #[tabled(rename = "IP")]
    ip: String,

    #[tabled(rename = "Discovered")]
    discovered: bool
}

#[derive(Tabled)]
struct ClientServerRow {
    #[tabled(rename = "Name")]
    name: String,

    #[tabled(rename = "Host")]
    host: String,

    #[tabled(rename = "Nodes")]
    num_nodes: String
}

pub fn print_node_table(nodes: &Vec<ClientNode>) {
    let rows: Vec<ClientNodeRow> = nodes.iter().map(|n| 
        ClientNodeRow {
            id: n.id.to_owned(),
            ip: n.ip.to_owned(),
            discovered: n.discovered.to_owned()
        }
    ).collect();

    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .with(
            Modify::new(Columns::last())
            .with(Alignment::center())
        );

    write_line!("{}", table);
}

pub fn print_server_table(servers: &Vec<ClientServer>, active_server_index: i32) {
    let rows: Vec<ClientServerRow> = servers.iter().map(|s| 
        ClientServerRow {
            name: s.name.to_owned(),
            host: s.host.to_owned(),
            num_nodes: s.nodes.len().to_string(),
        }
    ).collect();

    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .with(
            Modify::new(Columns::last())
            .with(Alignment::center())
        );

    if active_server_index != -1 {
        table.with(
            Modify::new(Cell::new(active_server_index as usize + 1, 0))
            .with(Color::BG_BRIGHT_GREEN)
        );
    }
        
    write_line!("{}", table);
}