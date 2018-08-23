
use holochain_dna::Dna;
use hash_table::entry::Entry;
use std::str::FromStr;
use holochain_agent::Agent;
use holochain_agent::Identity;
use serde_json;

pub trait ToEntry {
  fn to_entry(&self) -> Entry;
  // FIXME - Maybe change to `new_from_entry` ?
  fn from_entry(&Entry) -> Self;
}


//-------------------------------------------------------------------------------------------------
// Entry Type
//-------------------------------------------------------------------------------------------------

// Macro for statically concatanating the system entry prefix for entry types of system entries
macro_rules! sys_prefix {
    ($s:expr) => ( concat!("%", $s) )
}

// Enum for listing all System Entry Types
// Variant `Data` is for user defined entry types
#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
  AgentId,
  Deletion,
  Data,
  Dna,
  Headers,
  Key,
  Link,
  Migration,
}

impl FromStr for EntryType {
  type Err = usize;
  // Note: Function always return Ok()
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      sys_prefix!("agent_id") => Ok(EntryType::AgentId),
      sys_prefix!("deletion") => Ok(EntryType::Deletion),
      sys_prefix!("dna") => Ok(EntryType::Dna),
      sys_prefix!("headers") => Ok(EntryType::Headers),
      sys_prefix!("key") => Ok(EntryType::Key),
      sys_prefix!("link") => Ok(EntryType::Link),
      sys_prefix!("migration") => Ok(EntryType::Migration),
      _ => Ok(EntryType::Data),
    }
  }
}

impl EntryType {
  pub fn as_str(&self) -> &'static str {
    match *self {
      EntryType::Data =>      panic!("should not try to convert a custom data entry to str"),
      EntryType::AgentId =>   sys_prefix!("agent_id"),
      EntryType::Deletion =>  sys_prefix!("deletion"),
      EntryType::Dna =>       sys_prefix!("dna"),
      EntryType::Headers =>   sys_prefix!("headers"),
      EntryType::Key =>       sys_prefix!("key"),
      EntryType::Link =>      sys_prefix!("link"),
      EntryType::Migration => sys_prefix!("migration"),
    }
  }
}

//-------------------------------------------------------------------------------------------------
// Dna Entry
//-------------------------------------------------------------------------------------------------

impl ToEntry for Dna {
  fn to_entry(&self) -> Entry {
    Entry::new(
      EntryType::Dna.as_str(),
      &self.to_json().unwrap(),
    )
  }

  fn from_entry(entry: &Entry) -> Self {
    assert!(EntryType::from_str(&entry.entry_type()).unwrap() == EntryType::Dna);
    return Dna::new_from_json(&entry.content())
      .expect("entry is not a valid Dna Entry")
    ;
  }
}


//-------------------------------------------------------------------------------------------------
// Agent Entry
//-------------------------------------------------------------------------------------------------

impl ToEntry for Agent {
  fn to_entry(&self) -> Entry {
    Entry::new(
      EntryType::AgentId.as_str(),
      &self.to_string(),
    )
  }

  fn from_entry(entry: &Entry) -> Self {
    assert!(EntryType::from_str(&entry.entry_type()).unwrap() == EntryType::AgentId);
    let id_content : String = serde_json::from_str(&entry.content())
      .expect("entry is not a valid AgentId Entry");
    Agent::new(Identity::new(id_content))
  }
}


//-------------------------------------------------------------------------------------------------
// UNIT TESTS
//-------------------------------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {
  extern crate test_utils;

  use holochain_dna::Dna;
  use hash_table::entry::Entry;
  use holochain_agent::Agent;
  use holochain_agent::Identity;
  use action::ActionWrapper;
  use action::Action;
  use hash_table::sys_entry::EntryType;
  use hash_table::sys_entry::ToEntry;
  use std::{
    sync::{mpsc::channel, Arc, Mutex},
    str::FromStr,
  };

  use instance::{
    tests::{test_context, test_instance, test_instance_blank},
    Instance,
  };

  // Committing a DnaEntry to source chain should work
  #[test]
  fn can_commit_dna() {
    // Create Context, Agent, Dna, and Commit AgentIdEntry Action
    let context = test_context("alex");
    let mut dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
    let dna_entry = dna.to_entry();
    let commit_action = ActionWrapper::new(&Action::Commit(dna_entry));

    // Set up instance and dispatch action
    let mut instance = Instance::new();
    instance.start_action_loop(context);
    instance.dispatch_and_wait(&commit_action);

    // Check if AgentIdEntry is found
    let mut count = 0;
    instance.state().history.iter()
            .find(|aw| match aw.action() {
              Action::Commit(entry) => {
                assert_eq!(EntryType::from_str(&entry.entry_type()).unwrap(), EntryType::Dna);
                count += 1;
                true
              },
              _ => false,
            });
    assert_eq!(count, 1);

  }


  // Committing a DnaEntry to source chain should work only if first entry
  #[test]
  fn cannot_commit_dna() {
    // FIXME
  }


  // Committing an AgentIdEntry to source chain should work
  #[test]
  fn can_commit_agent() {
    // Create Context, Agent and Commit AgentIdEntry Action
    let context = test_context("alex");
    let agent_entry = context.agent.to_entry();
    let commit_agent_action = ActionWrapper::new(&Action::Commit(agent_entry));

    // Set up instance and dispatch action
    let mut instance = Instance::new();
    instance.start_action_loop(context);
    instance.dispatch_and_wait(&commit_agent_action);

    // Check if AgentIdEntry is found
    let mut count = 0;
    instance.state().history.iter()
      .find(|aw| match aw.action() {
        Action::Commit(entry) => {
          assert_eq!(EntryType::from_str(&entry.entry_type()).unwrap(), EntryType::AgentId);
          count += 1;
          true
        },
        _ => false,
      });
    assert_eq!(count, 1);
  }


  // Committing an AgentIdEntry to source chain should work only as second entry
  #[test]
  fn cannot_commit_agent() {
    // FIXME
  }
}