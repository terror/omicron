// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use id_map::Entry;
use id_map::IdMap;
use omicron_common::disk::DiskIdentity;
use sled_storage::disk::RawDisk;
use slog::Logger;
use tokio::sync::watch;

#[derive(Debug)]
pub struct RawDisksSender {
    disks: watch::Sender<IdMap<RawDisk>>,
}

impl RawDisksSender {
    pub fn new() -> (Self, watch::Receiver<IdMap<RawDisk>>) {
        let (disks, rx) = watch::channel(IdMap::default());
        (Self { disks }, rx)
    }

    pub fn set_raw_disks<I>(&self, raw_disks: I)
    where
        I: Iterator<Item = RawDisk>,
    {
        let new_disks = raw_disks.collect::<IdMap<_>>();
        self.disks.send_if_modified(|disks| {
            if *disks == new_disks {
                false
            } else {
                *disks = new_disks;
                true
            }
        });
    }

    pub fn add_or_update_raw_disk(&self, disk: RawDisk) {
        self.disks.send_if_modified(|disks| {
            match disks.entry(disk.identity().clone()) {
                Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(disk);
                    true
                }
                Entry::Occupied(mut occupied_entry) => {
                    if *occupied_entry.get() == disk {
                        false
                    } else {
                        occupied_entry.insert(disk);
                        true
                    }
                }
            }
        });
    }

    pub fn remove_raw_disk(&self, identity: &DiskIdentity, log: &Logger) {
        self.disks.send_if_modified(|disks| {
            let Some(disk) = disks.get(identity) else {
                info!(
                    log, "Ignoring request to remove nonexistent disk";
                    "identity" => ?identity,
                );
                return false;
            };

            if disk.is_synthetic() {
                // Synthetic disks are only added once; don't remove them.
                info!(
                    log, "Not removing synthetic disk";
                    "identity" => ?identity,
                );
                return false;
            }

            info!(log, "Removing disk"; "identity" => ?identity);
            disks.remove(identity);
            true
        });
    }
}
