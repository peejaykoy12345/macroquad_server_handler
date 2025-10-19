use std::any::Any;
use serde_json::{json,Value};

pub trait EntityTrait: Any + Send + Sync{
    fn to_json(&self,name:&str) -> Value{
        json!({})
    }
    fn is_colliding(&self, other: &dyn EntityTrait) -> bool{
        false
    }
    fn get_x_and_y(&self) -> (f32,f32){
        (0.0, 0.0)
    }
    fn get_position(&self) -> (f32,f32){
        self.get_x_and_y()
    }
    fn get_class(&self) -> String{
        String::from("EntityTrait")
    }
    fn get_mut_self(&mut self) -> &mut dyn Any;
}