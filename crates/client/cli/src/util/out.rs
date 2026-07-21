use tunnel_core::structs::client::ClientNode;
use tabled::{Table, Tabled, settings::{Alignment, Modify, Style, object::Columns}};

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