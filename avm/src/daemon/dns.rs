use std::str;

use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

use crate::daemon::daemon::{DaemonResult, ALAN_TECH_ENV};

pub struct DNS {
  resolver: TokioAsyncResolver,
  domain: String,
}

#[derive(Clone, Debug)]
pub struct VMMetadata {
  schema_version: String,
  pub(crate) private_ip_addr: String,
  pub(crate) alan_version: String,
  pub(crate) region: String,
  pub(crate) cloud: String,
}

impl VMMetadata {
  fn from_txt_data(data: &[u8]) -> DaemonResult<VMMetadata> {
    let txt = str::from_utf8(&*data);
    match txt {
      Ok(txt) => {
        let err = format!(
          "VM metadata in DNS TXT record has invalid schema version: `{}`",
          &txt
        );
        let parts: Vec<&str> = txt.split("|").collect();
        if parts.len() != 7 || parts[0] != "v1" {
          return Err(err.into());
        }
        Ok(VMMetadata {
          schema_version: parts[0].to_string(),
          alan_version: parts[1].to_string(),
          private_ip_addr: parts[4].to_string(),
          region: parts[5].to_string(),
          cloud: parts[6].to_string(),
        })
      }
      Err(_) => return Err("Data in TXT record is not a valid string".into()),
    }
  }

  fn get_local_metadata() -> DaemonResult<Vec<VMMetadata>> {
    Ok(vec![VMMetadata {
      schema_version: "v1".to_string(),
      alan_version: option_env!("CARGO_PKG_VERSION").unwrap().to_string(),
      cloud: "LOCAL".to_string(),
      private_ip_addr: "127.0.0.1".to_string(),
      region: "localhost".to_string(),
    }])
  }
}

impl DNS {
  pub fn new(domain: &str) -> DaemonResult<DNS> {
    let mut resolver_opts = ResolverOpts::default();
    // ignore /ect/hosts
    resolver_opts.use_hosts_file = false;
    // DNSSec
    resolver_opts.validate = true;
    // Get a new resolver with the cloudflare nameservers as the upstream recursive resolvers
    let resolver = ResolverConfig::cloudflare_tls();
    let resolver = TokioAsyncResolver::tokio(resolver, resolver_opts)?;
    Ok(DNS {
      domain: domain.to_string(),
      resolver: resolver,
    })
  }

  pub async fn get_vms(&self, cluster_id: &str) -> DaemonResult<Vec<VMMetadata>> {
    if ALAN_TECH_ENV.as_str() == "local" {
      return VMMetadata::get_local_metadata();
    };
    let name = format!("{}.{}", cluster_id, self.domain);
    let err = format!("Failed to fetch TXT record with name {}", &name);
    let resp = self.resolver.txt_lookup(name).await;
    let mut vms = Vec::new();
    if let Ok(resp) = resp {
      for rec in resp {
        let data = &rec.txt_data()[0];
        let vm = VMMetadata::from_txt_data(data)?;
        vms.push(vm);
      }
      Ok(vms)
    } else if let Err(err_resp) = resp {
      Err(format!("{}. {:?}", err, err_resp).into())
    } else {
      Err(err.into())
    }
  }
}
