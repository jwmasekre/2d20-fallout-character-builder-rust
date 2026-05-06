use glow::{Context, HasContext, NativeFramebuffer, NativeTexture, NativeProgram};

use crate::config::AppConfig;

pub struct CrtEffect {
    pub enabled: bool,
    pub tint: [f32; 3],
    pub distortion: f32,
    pub scanline_strength: f32,
    pub vignette_multiplier: f32,
    pub vignette_exponent: f32,
    pub roll_speed: f32,
    pub tint_strength: f32,
    pub chromatic_aberration: f32,
    start_time: std::time::Instant,
    fbo: NativeFramebuffer,
    fbo_texture: NativeTexture,
    program: NativeProgram,
    quad_vao: glow::NativeVertexArray,
    _quad_vbo: glow::NativeBuffer,
    width: i32,
    height: i32,
}

impl CrtEffect {
    pub fn new(
        gl: &Context,
        width: i32,
        height: i32,
    ) -> Self {
        unsafe {
            // --- FBO + texture ---
            let fbo = gl.create_framebuffer().unwrap();
            let fbo_texture = gl.create_texture().unwrap();

            gl.bind_texture(glow::TEXTURE_2D, Some(fbo_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0, glow::RGBA as i32,
                width, height, 0,
                glow::RGBA, glow::UNSIGNED_BYTE, None,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D, Some(fbo_texture), 0,
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            // --- Fullscreen quad ---
            // positions (xy) + uvs (uv) for a screen-filling triangle strip
            //let quad_verts: [f32; 24] = [
            let quad_verts: [f32; 16] = [
                -1.0,  1.0,  0.0, 1.0,
                -1.0, -1.0,  0.0, 0.0,
                 1.0,  1.0,  1.0, 1.0,
                 1.0, -1.0,  1.0, 0.0,
            ];

            let quad_vao = gl.create_vertex_array().unwrap();
            let _quad_vbo = gl.create_buffer().unwrap();
            gl.bind_vertex_array(Some(quad_vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(_quad_vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&quad_verts),
                glow::STATIC_DRAW,
            );
            // position attrib
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 16, 0);
            // uv attrib
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 16, 8);
            gl.bind_vertex_array(None);

            // --- Shader ---
            let program = compile_crt_shader(gl);

            let default_config = AppConfig::default();

            Self {
                enabled: true,
                tint: [1.0, 1.0, 1.0],
                distortion: default_config.crt_distortion,
                scanline_strength: default_config.crt_scanline_strength,
                vignette_multiplier: default_config.crt_vignette_multiplier,
                vignette_exponent: default_config.crt_vignette_exponent,
                roll_speed: default_config.crt_roll_speed,
                tint_strength: default_config.crt_tint_strength,
                chromatic_aberration: default_config.crt_chromatic_aberration,
                start_time: std::time::Instant::now(),
                fbo,
                fbo_texture,
                program,
                quad_vao,
                _quad_vbo,
                width,
                height }
        }
    }

    /// Call this BEFORE rendering ImGui — binds the FBO so everything renders into it
    pub fn begin_capture(&self, gl: &Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            gl.viewport(0, 0, self.width, self.height);
            gl.clear_color(0.05, 0.05, 0.05, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    /// Call this AFTER rendering ImGui — unbinds FBO and draws to screen
    pub fn end_capture_and_draw(&self, gl: &Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.viewport(0, 0, self.width, self.height);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            gl.use_program(Some(self.program));
            gl.bind_texture(glow::TEXTURE_2D, Some(self.fbo_texture));

            let _time = self.start_time.elapsed().as_secs_f32();

            for (name, val) in [
                ("uDistortion",        self.distortion),
                ("uScanlineStrength",  self.scanline_strength),
                ("uVignetteMult",      self.vignette_multiplier),
                ("uVignetteExp",       self.vignette_exponent),
                ("uRollSpeed",         self.roll_speed),
                ("uTintStrength",      self.tint_strength),
                ("uChromaticAberration", self.chromatic_aberration),
                ("uTime",              self.start_time.elapsed().as_secs_f32()),
            ]  {
                if let Some(loc) = gl.get_uniform_location(self.program, name) {
                    gl.uniform_1_f32(Some(&loc), val);
                }
            }
            if let Some(loc) = gl.get_uniform_location(self.program, "uTint") {
                gl.uniform_3_f32(Some(&loc), self.tint[0], self.tint[1], self.tint[2]);
            }

            gl.bind_vertex_array(Some(self.quad_vao));
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }

    /// Resize the FBO texture when the window changes size
    pub fn resize(&mut self, gl: &Context, width: i32, height: i32) {
        self.width = width;
        self.height = height;
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.fbo_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0, glow::RGBA as i32,
                width, height, 0,
                glow::RGBA, glow::UNSIGNED_BYTE, None,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }
}

unsafe fn compile_crt_shader(gl: &Context) -> NativeProgram {
    let vert_src = r#"
        #version 150 core
        in vec2 aPos;
        in vec2 aUv;
        out vec2 vUv;
        void main() {
            vUv = aUv;
            gl_Position = vec4(aPos, 0.0, 1.0);
        }
    "#;

    let frag_src = r#"
        #version 150 core
        in vec2 vUv;
        out vec4 fragColor;
        uniform sampler2D uScreen;
        uniform float uTime;
        uniform vec3 uTint;
        uniform float uDistortion;
        uniform float uScanlineStrength;
        uniform float uVignetteMult;
        uniform float uVignetteExp;
        uniform float uRollSpeed;
        uniform float uTintStrength;
        uniform float uChromaticAberration;

        // barrel distortion
        vec2 distort(vec2 uv) {
            vec2 cc = uv - 0.5;
            float dist = dot(cc, cc) * uDistortion;
            return uv + cc * dist;
        }

        void main() {
            vec2 uv = distort(vUv);

            // kill pixels outside the curved screen edge
            if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
                fragColor = vec4(0.0, 0.0, 0.0, 1.0);
                return;
            }

            vec4 col = texture(uScreen, uv);

            // subtle rgb shift (chromatic aberration)
            float shift = uChromaticAberration;
            col.r = texture(uScreen, vec2(uv.x + shift, uv.y)).r;
            col.b = texture(uScreen, vec2(uv.x - shift, uv.y)).b;

            // theme tint
            col.rgb *= mix(vec3(1.0), uTint, uTintStrength);

            // scanlines
            float scanline = sin(uv.y * 800.0) * uScanlineStrength;
            col.rgb -= scanline;

            // slow vertical roll line
            float roll = fract(uTime * uRollSpeed);
            float line = smoothstep(0.995, 1.0, fract(uv.y + roll));
            col.rgb += line * 0.04;
            // bigger line for testing
            //float roll = fract(uTime * 0.08);
            //float line = smoothstep(0.97, 1.0, fract(uv.y + roll));
            //col.rgb += line * 0.40;

            // vignette
            vec2 vig = uv * (1.0 - uv.yx);
            float vignette = pow(vig.x * vig.y * uVignetteMult, uVignetteExp);
            col.rgb *= vignette;

            fragColor = col;
        }
    "#;
    unsafe {
        let vert = gl.create_shader(glow::VERTEX_SHADER).unwrap();
        gl.shader_source(vert, vert_src);
        gl.compile_shader(vert);
        if !gl.get_shader_compile_status(vert) {
            panic!("CRT vert shader error: {}", gl.get_shader_info_log(vert));
        }

        let frag = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
        gl.shader_source(frag, frag_src);
        gl.compile_shader(frag);
        if !gl.get_shader_compile_status(frag) {
            panic!("CRT frag shader error: {}", gl.get_shader_info_log(frag));
        }

        let program = gl.create_program().unwrap();
        gl.attach_shader(program, vert);
        gl.attach_shader(program, frag);
        gl.bind_attrib_location(program, 0, "aPos");
        gl.bind_attrib_location(program, 1, "aUv");
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("CRT shader link error: {}", gl.get_program_info_log(program));
        }
        gl.delete_shader(vert);
        gl.delete_shader(frag);
        program
    }
}