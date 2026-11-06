//! Configuration for utilities that do not occupy HUD space.
use crate::{client_settings::{self, Utility, UtilityMod, Mods}, hud::{HudAtlas, HudBatch, rgba}, lang, text::Text, ui, widgets};
use winit::keyboard::KeyCode;

fn tr(key: &str) -> String { lang::get_or(key, key) }
#[derive(Clone, Copy, PartialEq)]
enum Field { Activation, Key, Factor, Smooth, Sensitivity, Weather, Strength, Shift }
impl Field {
    fn key(self, module: Utility) -> &'static str { match self { Self::Activation => "activation", Self::Key => "key", Self::Factor if module == Utility::Perspective => "camera_distance", Self::Factor => "zoom_factor", Self::Smooth => "smooth", Self::Sensitivity => "sensitivity", Self::Weather => "weather", Self::Strength => "intensity", Self::Shift => "require_shift" } }
    fn choices(self) -> &'static [&'static str] { match self { Self::Activation => &["hold", "toggle"], Self::Weather => &["clear", "rain", "thunder"], _ => &[] } }
    fn range(self, module: Utility) -> Option<(f32, f32)> { match self { Self::Factor => Some((1.5, if module == Utility::Perspective { 12.0 } else { 20.0 })), Self::Sensitivity | Self::Strength => Some((0.05, 1.0)), _ => None } }
    fn number(self, m: &UtilityMod) -> f32 { match self { Self::Factor => m.factor, Self::Sensitivity => m.sensitivity, Self::Strength => m.strength, _ => 0.0 } }
    fn value(self, m: &UtilityMod) -> String { match self { Self::Activation => tr(&format!("patina.value.{}", m.activation)), Self::Weather => tr(&format!("patina.value.{}", m.weather)), Self::Key => key_label(m.key), Self::Smooth | Self::Shift => tr(if if self == Self::Smooth { m.smooth } else { m.require_shift } { "patina.value.on" } else { "patina.value.off" }), _ => format!("{:.2}", self.number(m)) } }
}
fn key_label(code: i32) -> String { match code { -1 => tr("patina.value.unbound"), 32 => "Space".into(), 65..=90 | 48..=57 => char::from_u32(code as u32).unwrap().to_string(), 290..=314 => format!("F{}", code - 289), 340 => "Left Shift".into(), 341 => "Left Ctrl".into(), 342 => "Left Alt".into(), _ => format!("Key {code}") } }

pub(crate) struct UtilityScreen { pub module: Utility, binding: bool, dropdown: Option<Field>, slider: Option<(Field, [f32; 4])> }
impl UtilityScreen {
    pub fn new(module: Utility) -> Self { Self { module, binding: false, dropdown: None, slider: None } }
    fn fields(&self) -> Vec<Field> { use Field::*; match self.module { Utility::Zoom => vec![Activation, Key, Factor, Smooth, Sensitivity], Utility::Perspective => vec![Activation, Key, Factor], Utility::Weather => vec![Weather, Strength], Utility::ShulkerTooltip => vec![Shift], _ => vec![] } }
    fn rect(w: f32, h: f32) -> [f32; 4] { let pw = (w - 16.0).min(386.0).floor(); let ph = (h - 16.0).min(226.0).floor(); [((w-pw)/2.0).floor(), ((h-ph)/2.0).floor(), pw, ph] }
    fn rows(&self, w: f32, h: f32) -> Vec<(Field, [f32; 4])> { let [x,y,pw,_] = Self::rect(w,h); self.fields().into_iter().enumerate().map(|(i,f)| (f,[x+pw-112.0,y+50.0+i as f32*24.0,100.0,14.0])).collect() }
    pub fn escape(&mut self) -> bool { if self.binding { self.binding = false; true } else { self.dropdown.take().is_some() } }
    pub fn bind(&mut self, key: KeyCode) -> bool {
        if !self.binding { return false; }
        self.binding = false;
        // Delete unbinds; a key the shared table cannot name keeps the old binding.
        let code = if key == KeyCode::Delete { -1 } else { client_settings::glfw_key(key) };
        if key != KeyCode::Escape && (code >= 0 || key == KeyCode::Delete) { client_settings::update(|s| s.mods.utility_mut(self.module).key = code); }
        true
    }
    pub fn release(&mut self) { if self.slider.take().is_some() { client_settings::update(|_| {}); } }
    pub fn drag(&mut self, cursor: (f32, f32)) { if let Some((field, rect)) = self.slider { self.set_number(field, rect, cursor.0); } }
    fn set_number(&self, field: Field, [x,_,w,_]: [f32;4], cursor: f32) {
        let (min,max) = field.range(self.module).unwrap(); let value = min + ((cursor-x-4.0)/(w-8.0)).clamp(0.0,1.0)*(max-min);
        client_settings::update_unsaved(|s| { let m = s.mods.utility_mut(self.module); match field { Field::Factor => m.factor=value, Field::Sensitivity => m.sensitivity=value, Field::Strength => m.strength=value, _=>{} } });
    }
    /// None: no hit; Some(false): handled; Some(true): return to the catalogue.
    pub fn click(&mut self, cursor: (f32,f32), size: (f32,f32)) -> Option<bool> {
        let [x,y,w,h]=Self::rect(size.0,size.1); let hit=|r:[f32;4]|ui::hovered(cursor,r[0],r[1],r[2],r[3]);
        if let Some(field)=self.dropdown.take() {
            let (_,r)=self.rows(size.0,size.1).into_iter().find(|(f,_)|*f==field).unwrap();
            let top=(r[1]-field.choices().len() as f32*14.0).max(y+24.0);
            for (i,value) in field.choices().iter().enumerate() { if hit([r[0],top+i as f32*14.0,100.0,14.0]) { client_settings::update(|s| { let m=s.mods.utility_mut(self.module); match field { Field::Activation=>m.activation=(*value).into(),Field::Weather=>m.weather=(*value).into(),_=>{} } }); return Some(false); } }
            return Some(false);
        }
        if hit([x+w-62.0,y+6.0,50.0,14.0]) { return Some(true); }
        if hit([x+w-126.0,y+6.0,56.0,14.0]) { client_settings::update(|s| { let m=s.mods.utility_mut(self.module);m.enabled=!m.enabled; }); return Some(false); }
        if hit([x+12.0,y+h-24.0,96.0,15.0]) { client_settings::update(|s| { let enabled=s.mods.utility(self.module).enabled; *s.mods.utility_mut(self.module)=UtilityMod { enabled, key: if self.module==Utility::Perspective {86}else{67}, ..Default::default() }; });return Some(false); }
        for (field,rect) in self.rows(size.0,size.1) { if !hit(rect) {continue;} if field==Field::Key {self.binding=true;} else if !field.choices().is_empty(){self.dropdown=Some(field);} else if field.range(self.module).is_some(){self.slider=Some((field,rect));self.set_number(field,rect,cursor.0);} else {client_settings::update(|s|{let m=s.mods.utility_mut(self.module);if field==Field::Smooth {m.smooth=!m.smooth;}else{m.require_shift=!m.require_shift;}});} return Some(false); }
        None
    }
    pub fn draw(&self,batch:&mut HudBatch,atlas:&HudAtlas,mods:&Mods,cursor:(f32,f32)) {
        use crate::client_screen::{draw_label,draw_switch,reset_face};
        let (bw,bh)=(batch.width,batch.height); let [x,y,w,h]=Self::rect(bw,bh); let m=mods.utility(self.module);
        ui::plate(batch,atlas,[x,y,w,h],rgba(12,16,18,245),rgba(255,255,255,48));batch.fill(atlas,x,y,w,2.0,ui::ACCENT);
        let name=tr(&format!("patina.mod.{}",self.module.key())); draw_label(batch,atlas,&Text::literal(name),[x+12.0,y+6.0,w-148.0,14.0],rgba(111,220,195,255),0.9,false);
        let description=Text::literal(tr(&format!("patina.mod.{}.description",self.module.key())));
        for (i,line) in atlas.font.split(&description,(w-24.0)/0.72).into_iter().take(2).enumerate(){draw_label(batch,atlas,&line,[x+12.0,y+28.0+i as f32*8.0,w-24.0,8.0],rgba(159,176,172,255),0.72,false);}
        let back=[x+w-62.0,y+6.0,50.0,14.0];ui::button(batch,atlas,back[0],back[1],back[2],back[3],&Text::literal(lang::get_or("patina.client.back", "Back")),true,ui::hovered(cursor,back[0],back[1],back[2],back[3]));
        let toggle=[x+w-126.0,y+6.0,56.0,14.0];draw_switch(batch,atlas,toggle,m.enabled,ui::hovered(cursor,toggle[0],toggle[1],toggle[2],toggle[3]),0.82,false);widgets::record(&format!("Enabled: {}",m.enabled),toggle,true);
        for (field,r) in self.rows(bw,bh) {
            let label=tr(&format!("patina.option.{}",field.key(self.module)));let hover=ui::hovered(cursor,r[0],r[1],r[2],r[3]);draw_label(batch,atlas,&Text::literal(label.clone()),[x+12.0,r[1],w-140.0,r[3]],u32::MAX,0.82,false);
            if let Some((min,max))=field.range(self.module){ let value=((field.number(m)-min)/(max-min)).clamp(0.0,1.0);widgets::record_as(&label,r,true,widgets::Kind::Slider{value,track:[r[0]+4.0,r[2]-8.0]});let cx=r[0]+4.0+value*(r[2]-8.0);batch.fill(atlas,r[0]+4.0,r[1]+6.0,r[2]-8.0,2.0,rgba(74,84,84,255));batch.fill(atlas,cx-2.0,r[1]+4.0,4.0,6.0,ui::ACCENT);draw_label(batch,atlas,&Text::literal(field.value(m)),[r[0],r[1]-9.0,r[2],8.0],u32::MAX,0.72,true);}
            else if matches!(field,Field::Smooth|Field::Shift){draw_switch(batch,atlas,r,if field==Field::Smooth{m.smooth}else{m.require_shift},hover,0.82,false);widgets::record(&label,r,true);}
            else {let value=if field==Field::Key&&self.binding{tr("patina.option.press_key")}else{field.value(m)};ui::button(batch,atlas,r[0],r[1],r[2],r[3],&Text::literal(value),true,hover);widgets::record(&label,r,true);}
        }
        let reset=[x+12.0,y+h-24.0,96.0,15.0];let label=Text::literal(tr("patina.settings.reset"));reset_face(batch,atlas,reset,&label,0.82,ui::hovered(cursor,reset[0],reset[1],reset[2],reset[3]));widgets::record(&label.plain(),reset,true);
        if let Some(field)=self.dropdown{let(_,r)=self.rows(bw,bh).into_iter().find(|(f,_)|*f==field).unwrap();let top=(r[1]-field.choices().len() as f32*14.0).max(y+24.0);for(i,value)in field.choices().iter().enumerate(){let r=[r[0],top+i as f32*14.0,100.0,14.0];let hover=ui::hovered(cursor,r[0],r[1],r[2],r[3]);ui::plate(batch,atlas,r,if hover{rgba(31,57,53,255)}else{rgba(18,26,29,255)},ui::ACCENT);let value=tr(&format!("patina.value.{value}"));draw_label(batch,atlas,&Text::literal(value.clone()),r,u32::MAX,0.82,true);widgets::record(&value,r,true);}}
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn zoom_and_perspective_bind_independent_keys_without_changing_hud() {
        client_settings::update_unsaved(|s|*s=Default::default());
        let before=client_settings::get().mods.fps;
        let mut screen=UtilityScreen::new(Utility::Perspective);screen.binding=true;
        assert!(screen.bind(KeyCode::KeyB));assert_eq!(client_settings::get().mods.perspective.key,66);
        assert_eq!(client_settings::get().mods.zoom.key,67);assert_eq!(client_settings::get().mods.fps,before);
        screen.binding=true;screen.bind(KeyCode::Escape);assert_eq!(client_settings::get().mods.perspective.key,66);
        assert_eq!(UtilityScreen::new(Utility::Zoom).fields().len(),5);
    }
}
