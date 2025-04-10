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

    pub fn set_raw_disks<I>(&self, raw_disks: I, log: &Logger)
    where
        I: Iterator<Item = RawDisk>,
    {
        let new_disks = raw_disks.collect::<IdMap<_>>();
        self.disks.send_if_modified(|disks| {
            // We don't just set `*disks = new_disks` here because
            // `remove_raw_disk` has special logic to avoid removing synthetic
            // disks; instead, built up a list of "to remove", remove those
            // one-by-one, then add any new disks. (We could do this faster in
            // bulk here if we duplicated the check for synthetic disks, but
            // that doesn't really seem worth it over the cleaner separation of
            // logic.)
            let to_remove = disks
                .keys()
                .filter(|id| !new_disks.contains_key(id))
                .cloned()
                .collect::<Vec<_>>();

            let mut modified = false;

            for id in to_remove {
                let did_remove = Self::remove_raw_disk_inner(disks, &id, log);
                modified = modified || did_remove;
            }

            for disk in new_disks {
                let did_add_or_update =
                    Self::add_or_update_raw_disk_inner(disks, disk);
                modified = modified || did_add_or_update;
            }

            modified
        });
    }

    pub fn add_or_update_raw_disk(&self, disk: RawDisk) -> bool {
        self.disks.send_if_modified(|disks| {
            Self::add_or_update_raw_disk_inner(disks, disk)
        })
    }

    fn add_or_update_raw_disk_inner(
        disks: &mut IdMap<RawDisk>,
        disk: RawDisk,
    ) -> bool {
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
    }

    pub fn remove_raw_disk(
        &self,
        identity: &DiskIdentity,
        log: &Logger,
    ) -> bool {
        self.disks.send_if_modified(|disks| {
            Self::remove_raw_disk_inner(disks, identity, log)
        })
    }

    fn remove_raw_disk_inner(
        disks: &mut IdMap<RawDisk>,
        identity: &DiskIdentity,
        log: &Logger,
    ) -> bool {
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
    }
}
