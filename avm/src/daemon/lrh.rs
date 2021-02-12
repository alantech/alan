use twox_hash::XxHash64;

use std::hash::Hasher;

#[derive(Debug)]
struct HashedId {
  id: String,
  hash: u64,
}

impl HashedId {
  pub fn new(id: String) -> HashedId {
    let mut hasher = XxHash64::with_seed(0xfa57);
    hasher.write(id.as_bytes());
    HashedId {
      id,
      hash: hasher.finish(),
    }
  }
}

// An algorithm based on the classic Rendezvous Hash but with changes to make the performance
// closer to modern Consistent Hash algorithms while retaining Rendezvous Hash's coordination-free
// routing.
#[derive(Debug)]
pub struct LogRendezvousHash {
  sorted_hashes: Vec<HashedId>,
}

impl LogRendezvousHash {
  pub fn new(ids: Vec<String>) -> LogRendezvousHash {
    let mut sorted_hashes = Vec::with_capacity(ids.len());
    for id in ids {
      sorted_hashes.push(HashedId::new(id));
    }
    sorted_hashes.sort_by(|a, b| a.hash.cmp(&b.hash));
    LogRendezvousHash {
      sorted_hashes,
    }
  }

  pub fn get_leader_id(&self) -> &str {
    let last_idx = self.sorted_hashes.len() - 1;
    &self.sorted_hashes[last_idx].id 
  }

  // Runs a binary search for the record whose hash is closest to the key hash without
  // going over. If none are found, the *last* record in the list is returned as it wraps around.
  pub fn get_id_for_key(&self, key: &str) -> &str {
    let mut key_hasher = XxHash64::with_seed(0xfa57);
    key_hasher.write(key.as_bytes());
    let key_hash = key_hasher.finish();
    let idx = match self.sorted_hashes.binary_search_by(|a| a.hash.cmp(&key_hash)) {
      Ok(res) => res,
      // All were too large, implies last (which wraps around) owns it
      Err(_) => self.sorted_hashes.len() - 1,
    };
    &self.sorted_hashes[idx].id
  }
}