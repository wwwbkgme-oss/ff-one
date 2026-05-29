use crate::{abi::{FfPluginCtx,InfoFn,InitFn,ShutdownFn,TickFn,FF_ABI_VERSION},manifest::parse_manifest,registry::PluginRegistry};
use async_trait::async_trait;
use contracts::{error::{FfError,Result},traits::PluginHost};
use libloading::Library;
use std::collections::HashMap;
use tracing::{info,warn};
use types::plugin::{PluginCapability,PluginManifest,PluginRecord,PluginState};
struct Loaded{#[allow(dead_code)]record:PluginRecord,_lib:Library,tick_fn:Option<TickFn>,sd_fn:Option<ShutdownFn>}
pub struct PluginHostImpl{registry:PluginRegistry,loaded:HashMap<String,Loaded>,tick:u64}
impl PluginHostImpl{
    pub fn new()->Self{Self{registry:PluginRegistry::new(),loaded:HashMap::new(),tick:0}}
    pub fn tick_all(&mut self){self.tick+=1;for(id,p)in&self.loaded{if let Some(f)=p.tick_fn{let r=unsafe{f(self.tick)};if r!=0{warn!(plugin=%id,"non-zero tick");}}}}
    pub fn shutdown_all(&mut self){let ids:Vec<String>=self.loaded.keys().cloned().collect();for id in ids.iter().rev(){self.drop_one(id);}}
    fn drop_one(&mut self,id:&str){if let Some(p)=self.loaded.get(id){if let Some(f)=p.sd_fn{unsafe{f();}}}self.loaded.remove(id);self.registry.remove(id);}
}
impl Default for PluginHostImpl{fn default()->Self{Self::new()}}
#[async_trait]
impl PluginHost for PluginHostImpl{
    async fn load(&mut self,mp:&str)->Result<PluginRecord>{
        let manifest=parse_manifest(mp).map_err(|e|FfError::PluginLoadError(e.to_string()))?;
        self.validate_manifest(&manifest)?;
        let lp=manifest.lib.clone();
        info!(plugin=%manifest.id,"loading");
        let lib=unsafe{Library::new(&lp).map_err(|e|FfError::PluginLoadError(e.to_string()))?};
        let info_fn:InfoFn=unsafe{*lib.get::<InfoFn>(b"ff_plugin_info\0").map_err(|e|FfError::PluginLoadError(e.to_string()))?};
        let info=unsafe{&*info_fn()};
        if info.abi_version!=FF_ABI_VERSION{return Err(FfError::PluginLoadError(format!("ABI {}",info.abi_version)));}
        let init_fn:InitFn=unsafe{*lib.get::<InitFn>(b"ff_plugin_init\0").map_err(|e|FfError::PluginLoadError(e.to_string()))?};
        let ctx=FfPluginCtx{host:std::ptr::null(),load_tick:self.tick};
        let r=unsafe{init_fn(&ctx)};
        if r!=0{return Err(FfError::PluginLoadError(format!("init→{r}")));}
        let tick_fn:Option<TickFn>=unsafe{lib.get::<TickFn>(b"ff_plugin_tick\0").ok().map(|f|*f)};
        let sd_fn:Option<ShutdownFn>=unsafe{lib.get::<ShutdownFn>(b"ff_plugin_shutdown\0").ok().map(|f|*f)};
        let mut record=PluginRecord::new(manifest,&lp);
        record.state=PluginState::Initialised;
        self.registry.register(record.clone());
        self.loaded.insert(record.manifest.id.clone(),Loaded{record:record.clone(),_lib:lib,tick_fn,sd_fn});
        Ok(record)
    }
    async fn unload(&mut self,id:&str)->Result<()>{if !self.loaded.contains_key(id){return Err(FfError::PluginNotFound(id.to_string()));}self.drop_one(id);Ok(())}
    fn get(&self,id:&str)->Option<&PluginRecord>{self.registry.get(id)}
    fn list(&self)->Vec<&PluginRecord>{self.registry.iter().collect()}
    fn available_capabilities(&self)->Vec<PluginCapability>{self.registry.available_capabilities()}
    fn validate_manifest(&self,m:&PluginManifest)->Result<()>{let avail=self.available_capabilities();for req in&m.requires{if !avail.contains(req){return Err(FfError::UnsatisfiedDependency(m.id.clone(),req.to_string()));}}Ok(())}
}
