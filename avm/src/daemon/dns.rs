use std::error::Error;
use std::str;

use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

pub struct DNS {
  resolver: TokioAsyncResolver,
  domain: String,
}

pub struct VMMetadata {
  schema_version: String,
  pub(crate) private_ip_addr: String,
  pub(crate) alan_version: String,
}

impl VMMetadata {
  fn from_txt_data(data: &[u8]) -> VMMetadata {
    let txt = str::from_utf8(&*data).expect("Data in TXT record is not a valid string");
    let err = format!(
      "VM metadata in DNS TXT record has invalid schema version: `{}`",
      &txt
    );
    let parts: Vec<&str> = txt.split("|").collect();
    if parts.len() != 5 || parts[0] != "v1" {
      panic!(err);
    }
    VMMetadata {
      schema_version: parts[0].to_string(),
      alan_version: parts[1].to_string(),
      private_ip_addr: parts[4].to_string(),
    }
  }
}

impl DNS {
  pub fn new(domain: &str) -> Result<DNS, Box<dyn Error>> {
    let mut resolver_opts = ResolverOpts::default();
    // ignore /ect/hosts
    resolver_opts.use_hosts_file = false;
    // DNSSec
    resolver_opts.validate = true;
    // Get a new resolver with the cloudflare nameservers as the upstream recursive resolvers
    let resolver = ResolverConfig::cloudflare_tls();
    let resolver_result = TokioAsyncResolver::tokio(resolver, resolver_opts);
    match resolver_result {
      Ok(resolver) => Ok(DNS {
        domain: domain.to_string(),
        resolver: resolver,
      }),
      Err(e) => Err(e.into()),
    }
  }

  pub async fn get_vms(&self, cluster_id: &str) -> Result<Vec<VMMetadata>, Box<dyn Error>> {
    let name = format!("{}.{}", cluster_id, self.domain);
    let err = format!("Failed to fetch TXT record with name {}", &name);
    let resp = self.resolver.txt_lookup(name).await;
    let mut vms = Vec::new();
    if let Ok(resp) = resp {
      for rec in resp {
        let data = &rec.txt_data()[0];
        let vm = VMMetadata::from_txt_data(data);
        vms.push(vm);
      }
      Ok(vms)
    } else {
      return Err(err.into());
    }
    
  }
}
