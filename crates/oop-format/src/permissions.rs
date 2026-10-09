//! The permissions a plugin can declare in its header.
//!
//! Reading the game's state needs no permission; everything that changes it, shows something,
//! keeps data or talks to other programs does. The game checks a call against the header's list
//! (plain `.lua` files keep full access for compatibility). The list is open: a later openOMSI
//! may add names, and an unknown name is not an error here (`oopc check` warns about it).

/// A permission the tools know, with the sentence the devtools show next to its checkbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permission {
    /// The name in the header: `vehicle_write`.
    pub name: &'static str,
    /// What it allows, for people deciding whether to trust a plugin.
    pub description: &'static str,
}

/// Every permission this version of the tools knows: the game's own list (openOMSI's
/// `crates/omsi-plugin/src/api` `Perm`, and each function's `permission` in its
/// `docs/plugin-api.json`).
pub const KNOWN: &[Permission] = &[
    Permission { name: "ui", description: "Show panels, notifications and messages on the screen" },
    Permission { name: "storage", description: "Keep data between sessions in the plugin's own data folder and storage" },
    Permission {
        name: "vehicle_write",
        description: "Change the player's bus: script variables, keys and triggers, doors, the destination",
    },
    Permission {
        name: "traffic_write",
        description: "Change the AI traffic and the people, and other vehicles' script variables",
    },
    Permission {
        name: "world_write",
        description: "Change the world: weather, time, pause, teleport, the duty and the game menu's actions (refuel, repair, save, ...)",
    },
    Permission { name: "camera", description: "Move the camera and switch the view" },
    Permission { name: "audio", description: "Play sounds" },
    Permission {
        name: "network_local",
        description: "Send messages to programs on this computer (UDP to 127.0.0.1, never the internet)",
    },
    Permission { name: "lan", description: "Send messages to the other players of a LAN game" },
];

/// Looks a permission up by name.
pub fn find(name: &str) -> Option<&'static Permission> {
    KNOWN.iter().find(|p| p.name == name)
}
