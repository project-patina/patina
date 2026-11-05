//! AppleSkin's food overlays using the upstream 3.0.10 icon sheet (Unlicense).
//! Food values remain read-only; offsets come from the same vanilla HUD layout.
use crate::{hud::{HudAtlas,HudBatch,Rect,rgba},hud_layout::{GuiDraw,HealthInputs}};

fn texture(batch:&mut HudBatch,atlas:&HudAtlas,at:[f32;2],uv:[f32;4],alpha:f32) {
    let Some(r)=atlas.sprites.get("patina/appleskin").copied()else{return;};
    let [u,v,w,h]=uv;let du=(r.u1-r.u0)/256.0;let dv=(r.v1-r.v0)/256.0;
    let sprite=Rect{u0:r.u0+u*du,v0:r.v0+v*dv,u1:r.u0+(u+w)*du,v1:r.v0+(v+h)*dv,..r};
    batch.rect_uv(at[0],at[1],w,h,sprite,rgba(255,255,255,(alpha.clamp(0.0,1.0)*255.0)as u8));
}
fn foods(draws:&[GuiDraw])->Vec<(f32,f32)> {draws.iter().filter_map(|d|match d{GuiDraw::Sprite{sprite,x,y,..}if sprite.starts_with("hud/food_empty")=>Some((*x as f32,*y as f32)),_=>None}).collect()}
fn sprite(batch:&mut HudBatch,atlas:&HudAtlas,key:&str,x:f32,y:f32,alpha:f32) {if let Some(r)=atlas.sprites.get(&format!("gui/sprites/{key}")).copied(){batch.rect_uv(x,y,9.0,9.0,r,rgba(255,255,255,(alpha*255.0)as u8));}}
fn saturation(batch:&mut HudBatch,atlas:&HudAtlas,positions:&[(f32,f32)],added:f32,current:f32,alpha:f32){
    let total=(current+added).max(0.0);let start=if added==0.0{0}else{(current/2.0).max(0.0)as usize};
    for i in start..((total/2.0).ceil()as usize).min(positions.len()){let amount=total/2.0-i as f32;let u=if amount>=1.0{27.0}else if amount>0.5{18.0}else if amount>0.25{9.0}else{0.0};let(x,y)=positions[i];texture(batch,atlas,[x,y],[u,0.0,9.0,9.0],alpha);}
}
/// Same exhaustion, saturation and held-food preview placement as upstream.
pub fn underlay(batch:&mut HudBatch,atlas:&HudAtlas,draws:&[GuiDraw],exhaustion:f32){
    let positions=foods(draws);let Some(&(x,y))=positions.first()else{return;};let width=(exhaustion/4.0).clamp(0.0,1.0)*81.0;let width=width.floor();texture(batch,atlas,[x+9.0-width,y],[81.0-width,18.0,width,9.0],0.75);
}
#[derive(Clone,Copy)]
pub struct HeldFood {pub nutrition:u32,pub saturation:f32,pub regeneration:f32,pub rotten:bool}
pub fn overlay(batch:&mut HudBatch,atlas:&HudAtlas,draws:&[GuiDraw],input:&HealthInputs,held:Option<HeldFood>,exhaustion:f32,natural_regeneration:bool){
    let positions=foods(draws);saturation(batch,atlas,&positions,0.0,input.saturation,1.0);
    let Some(HeldFood{nutrition,saturation:sat,regeneration,rotten})=held else{return;};
    let alpha=((input.millis as f32/1250.0)%2.0-1.0).abs()*0.65;
    let restored=(input.food+nutrition as i32).min(20);let new_sat=(input.saturation+sat).min(restored as f32);
    for i in (input.food.max(0)/2)as usize..((restored+1)/2).max(0)as usize{if let Some(&(x,y))=positions.get(i){sprite(batch,atlas,if rotten{"hud/food_empty_hunger"}else{"hud/food_empty"},x,y,alpha);sprite(batch,atlas,if rotten{if i as i32*2+1==restored{"hud/food_half_hunger"}else{"hud/food_full_hunger"}}else if i as i32*2+1==restored{"hud/food_half"}else{"hud/food_full"},x,y,alpha);}}
    saturation(batch,atlas,&positions,new_sat-input.saturation,input.saturation,alpha);
    if input.health<input.max_health {
        let health=(input.health+regeneration+if natural_regeneration{estimated_health(restored,new_sat,exhaustion)}else{0.0}).min(input.max_health);
        let positions:Vec<_>=draws.iter().filter_map(|d|match d{GuiDraw::Sprite{sprite,x,y,..}if sprite.starts_with("hud/heart/container")=>Some((*x as f32,*y as f32)),_=>None}).collect();
        // Vanilla lays hearts out from the last heart down to the first.
        let mut positions=positions;positions.sort_by(|a,b|b.1.total_cmp(&a.1).then(a.0.total_cmp(&b.0)));positions.dedup();
        for i in (input.health.ceil()as usize/2)..(health.ceil()as usize).div_ceil(2){if let Some(&(x,y))=positions.get(i){let kind=if input.hardcore{"hardcore_"}else{""};let fill=if i as i32*2+1==health.ceil()as i32{"half"}else{"full"};sprite(batch,atlas,&format!("hud/heart/{kind}{fill}"),x,y,alpha);}}
    }
}
/// Upstream FoodHelper.getEstimatedHealthIncrement(int,float,float).
pub fn estimated_health(mut food:i32,mut saturation:f32,mut exhaustion:f32)->f32 {
    if !saturation.is_finite()||!exhaustion.is_finite(){return 0.0;}
    let mut health=0.0;
    for _ in 0..10000 {if food<18{break;}while exhaustion>4.0{exhaustion-=4.0;if saturation>0.0{saturation=(saturation-1.0).max(0.0);}else{food-=1;}}
        if food>=20&&saturation>f32::MIN_POSITIVE{let increment=saturation.min(6.0);let count=((f32::from_bits(4.0f32.to_bits()+1)-exhaustion)/increment).ceil().max(1.0);health+=increment/6.0*count;exhaustion+=increment*count;}
        else if food>=18{health+=1.0;exhaustion+=6.0;}
    }
    health
}
pub fn food_tooltip(batch:&mut HudBatch,atlas:&HudAtlas,x:f32,y:f32,nutrition:u32,sat:f32){
    let count=nutrition.div_ceil(2).min(10);for i in 0..count {sprite(batch,atlas,if i*2+1==nutrition{"hud/food_half"}else{"hud/food_full"},x+i as f32*9.0,y,1.0);}
    let positions:Vec<_>=(0..10).map(|i|(x+i as f32*9.0,y+10.0)).collect();saturation(batch,atlas,&positions,0.0,sat,1.0);
}

#[cfg(test)]mod tests{use super::*;#[test]fn health_estimate_stops_when_food_cannot_regenerate(){assert_eq!(estimated_health(17,0.0,0.0),0.0);assert!(estimated_health(20,5.0,0.0)>estimated_health(20,0.0,0.0));assert_eq!(estimated_health(20,f32::NAN,0.0),0.0);}}
