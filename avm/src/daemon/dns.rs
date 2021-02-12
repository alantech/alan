use std::str;

use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

pub struct DNS {
  resolver: TokioAsyncResolver,
  domain: String
}

impl DNS {
  pub fn new(domain: &str) -> DNS {
    // Get a new resolver with the cloudflare nameservers as the upstream recursive resolvers
    DNS {
      domain: domain.to_string(),
      resolver: TokioAsyncResolver::tokio(
        ResolverConfig::cloudflare(),
        ResolverOpts::default(),
      ).unwrap()
    }
  }
  pub async fn get_ip_addrs(&self, app_id: &str) -> Vec<String> {
    let name = format!("{}.{}", app_id, self.domain);
    let err = format!("Failed to fetch TXT record with name {}", &name);
    let resp = self.resolver.txt_lookup(name).await;
    let mut vm_ids = Vec::new();
    for rec in resp.expect(&err) {
      let data = &rec.txt_data()[0];
      let vm_id = DNS::ip_from_txt_data(data);
      vm_ids.push(vm_id);
    }
    vm_ids
  }

  fn ip_from_txt_data(data: &[u8]) -> String {
    let txt = str::from_utf8(&*data).expect("Data in TXT record is not a valid string");
    let err = format!("VM metadata in DNS TXT record has invalid schema version: `{}`", &txt);
    let parts: Vec<&str> = txt.split("|").collect();
    if parts.len() != 4 || parts[0] != "v1" {
      panic!(err);
    }
    parts[3].to_string()
  }
}
