use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct FloatBuffer {
    values: Vec<f32>,
}

impl FloatBuffer {
    pub fn replace(&mut self, values: Vec<f32>) {
        self.values = values;
    }

    pub fn values(&self) -> &[f32] {
        &self.values
    }
}

#[wasm_bindgen]
impl FloatBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn push(&mut self, value: f32) {
        self.values.push(value);
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn view(&self) -> js_sys::Float32Array {
        unsafe { js_sys::Float32Array::view(&self.values) }
    }
}

impl Default for FloatBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
pub struct UIntBuffer {
    values: Vec<u32>,
}

impl UIntBuffer {
    pub fn replace(&mut self, values: Vec<u32>) {
        self.values = values;
    }

    pub fn values(&self) -> &[u32] {
        &self.values
    }
}

#[wasm_bindgen]
impl UIntBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn view(&self) -> js_sys::Uint32Array {
        unsafe { js_sys::Uint32Array::view(&self.values) }
    }
}

impl Default for UIntBuffer {
    fn default() -> Self {
        Self::new()
    }
}
