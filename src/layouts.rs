use penrose::core::{
    data_types::{Region, ResizeAction},
    Client, Layout, LayoutConf,
};
use std::collections::HashMap;

use crate::CLIENT_REGIONS;

fn main_plus_side(
    clients: &[&Client],
    _active: Option<u32>,
    monitor_region: &Region,
    _in_main: u32,
    ratio: f32,
) -> Vec<ResizeAction> {
    let resize_actions = match clients.len() {
        0 => vec![],
        1 => return vec![(clients[0].id(), Some(*monitor_region))],
        n => {
            let (main, stack) = monitor_region
                .split_at_width(((monitor_region.w as f32) * ratio) as u32)
                .expect("Given ration was invalid");

            let stack_rows = stack.as_rows(n as u32 - 1);

            vec![main]
                .into_iter()
                .chain(stack_rows)
                .zip(clients)
                .map(|(r, c)| (c.id(), Some(r)))
                .collect()
        }
    };

    *CLIENT_REGIONS
        .write()
        .expect("CLIENT_REGIONS RwLock poisoned") = resize_actions
        .clone()
        .into_iter()
        .map(|(id, region)| (id, region.expect("Region was None")))
        .collect::<HashMap<_, _>>();

    resize_actions
}

fn dwindle(
    clients: &[&Client],
    _active: Option<u32>,
    monitor_region: &Region,
    _in_main: u32,
    _ratio: f32,
) -> Vec<ResizeAction> {
    enum Split {
        Horizontal,
        Vertical,
    }

    if clients.is_empty() {
        return vec![];
    }

    let mut clients = clients.to_vec();

    let mut split = if monitor_region.w > monitor_region.h {
        Split::Vertical
    } else {
        Split::Horizontal
    };

    let mut actions = vec![(clients.remove(0).id(), *monitor_region)];

    for client in clients {
        match split {
            Split::Horizontal => {
                let last_client = actions.last_mut().unwrap_or_else(|| unreachable!());

                let (top, bottom) = last_client
                    .1
                    .split_at_height(last_client.1.h / 2)
                    .expect("Invalid region");

                last_client.1 = top;

                actions.push((client.id(), bottom));

                split = Split::Vertical;
            }
            Split::Vertical => {
                let last_client = actions.last_mut().unwrap_or_else(|| unreachable!());

                let (left, right) = last_client
                    .1
                    .split_at_width(last_client.1.w / 2)
                    .expect("Invalid region");

                last_client.1 = left;

                actions.push((client.id(), right));

                split = Split::Horizontal;
            }
        }
    }

    actions
        .into_iter()
        .map(|(id, region)| (id, Some(region)))
        .collect()
}
#[must_use]
pub fn layouts() -> Vec<Layout> {
    vec![
        Layout::new("main+side", LayoutConf::default(), main_plus_side, 1, 0.8),
        Layout::new("dwindle", LayoutConf::default(), dwindle, 0, 0.),
    ]
}
