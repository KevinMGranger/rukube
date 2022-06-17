use anyhow::bail;
use console::style;
use dialoguer::Confirm;

use std::collections::{BTreeSet, HashSet};
use std::fmt::Write as _;
use std::fs;

use tabular::{row, Table};

use rustkube::{read_config, kube_dir, write_config};
use chrono::{Local};

// struct KubeConfigEdit<'a> {
//     clusters: &'a HashMap<String, ClusterSpec>,
//     contexts: &'a HashMap<String, ContextSpec>,
//     users: &'a HashMap<String, UserSpec>,
// }

// impl<'a> fmt::Display for KubeConfigEdit<'a> {
//     fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
//         writeln!(w, "Clusters:\n")?;
//         for cluster in self.clusters.keys() {
//             writeln!(w, "{cluster}")?;
//         }
//         writeln!(w, "\nContexts:\n")?;
//         let mut table = Table::new("{:<} {:<} {:<}");
//         for (ctx_name, ctx_spec) in self.contexts {
//             let user = &ctx_spec.user;
//             let cluster = &ctx_spec.cluster;

//             table.add_row(row!(
//                 ctx_name,
//                 format!("🖥  {cluster}"),
//                 format!("🧑 {user}")
//             ));
//         }
//         writeln!(w, "{table}")?;
//         writeln!(w, "\nUsers:\n")?;
//         for user_name in self.users.keys() {
//             writeln!(w, "{user_name}")?;
//         }
//         Ok(())
//     }
// }

// fn write_info<'a>(
//     mut w: impl io::Write,
//     clusters: impl Iterator<Item = &'a str>,
//     contexts: &HashMap<String, ContextSpec>,
//     users: impl Iterator<Item = &'a str>,
// ) -> io::Result<()> {
//     writeln!(w, "Clusters:\n")?;
//     for cluster in clusters {
//         writeln!(w, "{cluster}")?;
//     }
//     writeln!(w, "\nContexts:\n")?;
//     let mut table = Table::new("{:<} {:<} {:<}");
//     for (ctx_name, ctx_spec) in contexts {
//         let user = &ctx_spec.user;
//         let cluster = &ctx_spec.cluster;

//         table.add_row(row!(
//             ctx_name,
//             format!("🖥  {cluster}"),
//             format!("🧑 {user}")
//         ));
//     }
//     writeln!(w, "{table}")?;
//     writeln!(w, "\nUsers:\n")?;
//     for user_name in users {
//         writeln!(w, "{user_name}")?;
//     }
//     Ok(())
// }

enum EditResult<T> {
    Keep(T),
    Remove(T),
}

fn main() -> anyhow::Result<()> {
    let mut kc = read_config()?;

    // region: Clusters
    // TODO: if the cluster name has a tab in it this will break
    let mut cluster_list = String::new();
    for (cluster_name, cluster_spec) in &kc.clusters {
        let server = &cluster_spec.server;
        write!(&mut cluster_list, "{cluster_name}\t({server})\n")?;
    }

    let edited = dialoguer::Editor::new()
        .edit(&cluster_list)
        .unwrap()
        .unwrap_or(cluster_list);

    let remaining = edited
        .lines()
        .map(|line| line.split('\t').next().unwrap())
        .collect::<BTreeSet<_>>();

    let cluster_ops = kc.clusters.keys().map(|cluster| {
        if remaining.contains(cluster.as_str()) {
            EditResult::Keep(cluster)
        } else {
            EditResult::Remove(cluster)
        }
    });

    let mut remaining_clusters = kc.clusters.clone();
    remaining_clusters.retain(|k, _| remaining.contains(k.as_str()));
    // endregion

    let mut remaining_contexts = kc.contexts.clone();
    remaining_contexts.retain(|_, ctx| remaining.contains(ctx.cluster.as_str()));

    let current = &kc.current_context;
    if !remaining_contexts.contains_key(current) {
        bail!("ERROR: default context ({current}), would be removed");
    }

    let context_ops = kc.contexts.iter().map(|(name, ctx)| {
        if remaining_contexts.contains_key(name) {
            EditResult::Keep((name, ctx))
        } else {
            EditResult::Remove((name, ctx))
        }
    });

    let remaining_user_names = remaining_contexts
        .values()
        .map(|ctx| ctx.user.as_str())
        .collect::<HashSet<_>>();

    let mut remaining_users = kc.users.clone();
    remaining_users.retain(|name, _| remaining_user_names.contains(name.as_str()));

    let user_ops = kc.users.keys().map(|user| {
        if remaining_user_names.contains(user.as_str()) {
            EditResult::Keep(user)
        } else {
            EditResult::Remove(user)
        }
    });

    println!("Clusters:");
    for cluster in cluster_ops {
        match cluster {
            EditResult::Keep(cluster) => println!("{} {}", ' ', cluster),
            EditResult::Remove(cluster) => {
                println!("{} {}", style('-').red(), style(cluster).red())
            }
        }
    }
    println!("\nContexts:");
    let mut table = Table::new("{:<} {:<} {:<} {:<}");
    for context_op in context_ops {
        let (sym, name, cluster, user) = match context_op {
            EditResult::Keep((name, ctx)) => (' ', name, &ctx.cluster, &ctx.user),
            EditResult::Remove((name, ctx)) => ('-', name, &ctx.cluster, &ctx.user),
        };
        table.add_row(row!(
            sym,
            name,
            format!("🖥  {cluster}"),
            format!("🧑 {user}")
        ));
    }
    for line in table.to_string().lines() {
        if line.chars().next().unwrap() == '-' {
            println!("{}", style(line).red())
        } else {
            println!("{line}")
        };
    }

    println!("\nUsers:");
    for user in user_ops {
        match user {
            EditResult::Keep(user) => println!("  {user}"),
            EditResult::Remove(user) => println!("{} {}", style('-').red(), style(user).red()),
        }
    }

    if !Confirm::new().with_prompt("Perform edit?").wait_for_newline(true).interact().unwrap() {
        return Ok(());
    }

    kc.clusters = remaining_clusters;
    kc.contexts = remaining_contexts;
    kc.users = remaining_users;

    let kube_dir = kube_dir();

    let kube_config_path = kube_dir.join("config");

    let now = Local::now().naive_local();

    let current_backup = kube_dir.join(format!("config_{now}"));

    fs::rename(&kube_config_path, &current_backup)?;

    write_config(kc, &kube_config_path)?;

    Ok(())
}
