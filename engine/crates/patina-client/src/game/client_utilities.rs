//! Built-in client utilities. Visual overrides never write weather or player rotation to the level.
use super::{Game, Screen, perspective::CameraType};
use patina_client::{client_settings::{self, UtilityMod}, clipboard::Clipboard, hud::{HudBatch, rgba}};
use std::time::Duration;

pub(super) struct UtilityState {
    pub zoom: bool, pub perspective: bool, pub yaw: f32, pub pitch: f32,
    pub zoom_factor: f32, pub current_zoom: f32,
    zoom_press: bool, perspective_press: bool, previous_camera: Option<CameraType>,
    was_dead: bool, pub death: Option<(String,String,String)>,
}
impl Default for UtilityState {
    fn default()->Self { Self {zoom:false,perspective:false,yaw:0.0,pitch:0.0,zoom_factor:4.0,current_zoom:1.0,zoom_press:false,perspective_press:false,previous_camera:None,was_dead:false,death:None} }
}
/// Hold: on while the key is down. Toggle: `down` is a press since the last frame, and flips it.
pub(super) fn activation(m:&UtilityMod, playing:bool, down:bool, active:bool)->bool {
    m.enabled && playing && if m.activation=="toggle" { if down {!active}else{active} }else{down}
}
impl Game<'_> {
    pub(super) fn utility_key_press(&mut self,key:winit::keyboard::KeyCode) {
        if self.screen!=Screen::Playing||self.title.is_some()||self.player.is_none(){return;}
        let code=client_settings::glfw_key(key);let mods=client_settings::get().mods;
        if mods.zoom.enabled&&mods.zoom.activation=="toggle"&&mods.zoom.key==code {self.extras.utilities.zoom_press=!self.extras.utilities.zoom_press;}
        if mods.perspective.enabled&&mods.perspective.activation=="toggle"&&mods.perspective.key==code {self.extras.utilities.perspective_press=!self.extras.utilities.perspective_press;}
    }
    pub(super) fn utility_frame(&mut self, elapsed:Duration) {
        let mods=client_settings::get().mods;
        let playing=self.player.is_some()&&self.title.is_none()&&self.screen==Screen::Playing&&(self.focused||self.control.is_some());
        let key=|code|code>=0&&self.keys.iter().any(|key|client_settings::glfw_key(*key)==code);
        let (zk,pk)=(key(mods.zoom.key),key(mods.perspective.key));
        let state=&mut self.extras.utilities;
        let zoom=activation(&mods.zoom,playing,if mods.zoom.activation=="toggle"{state.zoom_press}else{zk},state.zoom);
        let perspective=activation(&mods.perspective,playing,if mods.perspective.activation=="toggle"{state.perspective_press}else{pk},state.perspective);
        if zoom&&!state.zoom {state.zoom_factor=mods.zoom.factor;}
        if perspective&&!state.perspective {state.previous_camera=Some(self.camera_type);state.yaw=self.yaw;state.pitch=self.pitch;self.camera_type=CameraType::ThirdPersonBack;}
        else if !perspective&&state.perspective&&let Some(previous)=state.previous_camera.take(){self.camera_type=previous;}
        state.zoom=zoom;state.perspective=perspective;state.zoom_press=false;state.perspective_press=false;
        let target=if zoom{state.zoom_factor}else{1.0};
        state.current_zoom=if mods.zoom.smooth{state.current_zoom+(target-state.current_zoom)*(1.0-(-elapsed.as_secs_f32().min(0.1)*16.0).exp())}else{target};
        if (state.current_zoom-target).abs()<0.001 {state.current_zoom=target;}
        let dead=self.screen==Screen::Dead;
        if dead&&!state.was_dead&&mods.death_info.enabled&&let Some(player)=&self.player {
            let coordinates=format!("{} {} {}",player.position.x.floor()as i64,player.position.y.floor()as i64,player.position.z.floor()as i64);
            let dimension=match self.dim {super::Dim::Overworld=>"minecraft:overworld",super::Dim::Nether=>"minecraft:the_nether",super::Dim::End=>"minecraft:the_end"}.to_owned();
            let time=chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            self.clipboard.set(&coordinates);
            state.death=Some((coordinates.clone(),dimension.clone(),time.clone()));
            if let Ok(dir)=std::env::current_dir(){let folder=dir.join("patina");let _=std::fs::create_dir_all(&folder);let data=serde_json::json!({"coordinates":coordinates,"dimension":dimension,"time":time});let _=std::fs::write(folder.join("last-death.json"),data.to_string());}
        }
        state.was_dead=dead;
    }
    pub(super) fn utility_zoom_wheel(&mut self,amount:f32)->bool {
        if !self.extras.utilities.zoom||self.screen!=Screen::Playing{return false;}
        self.extras.utilities.zoom_factor=(self.extras.utilities.zoom_factor*1.15f32.powf(amount)).clamp(1.5,20.0);true
    }
    pub(super) fn visual_weather(&self,partial:f32)->(f32,f32) {
        let weather=client_settings::get().mods.weather;
        if !weather.enabled {return self.world.weather.levels(partial);}
        match weather.weather.as_str(){"rain"=>(weather.strength,0.0),"thunder"=>(weather.strength,weather.strength),_=>(0.0,0.0)}
    }
    pub(super) fn draw_death_info(&self,batch:&mut HudBatch) {
        if !client_settings::get().mods.death_info.enabled{return;}
        let Some((coordinates,dimension,time))=&self.extras.utilities.death else{return;};
        let atlas=&self.gui.atlas;
        for(i,text)in [dimension.clone(),time.clone(),format!("{coordinates} — {}",patina_client::lang::get_or("patina.death.copied","copied"))].iter().enumerate(){let width=HudBatch::text_width(atlas,text);batch.text(atlas,text,((batch.width-width)/2.0).floor(),(batch.height/4.0).floor()+126.0+i as f32*10.0,if i==2{rgba(111,220,195,255)}else{u32::MAX},true);}
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn activation_modes_require_a_press_and_stop_when_disabled_or_a_screen_opens(){
        let mut m=UtilityMod{enabled:true,..Default::default()};assert!(activation(&m,true,true,false));assert!(!activation(&m,true,false,true));m.activation="toggle".into();assert!(activation(&m,true,true,false));assert!(activation(&m,true,false,true));assert!(!activation(&m,true,true,true));assert!(!activation(&m,false,false,true));m.enabled=false;assert!(!activation(&m,true,true,true));
    }
}
