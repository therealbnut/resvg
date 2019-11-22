use std::ops::{Deref, DerefMut};
use std::cell::RefCell;

use skia_safe::*;

pub use skia_bindings::SkSurface as skiac_surface;
pub use skia_bindings::SkCanvas as skiac_canvas;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PaintStyle {
    Fill = 0,
    Stroke = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FillType {
    Winding = 0,
    EvenOdd = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StrokeCap {
    Butt = 0,
    Round = 1,
    Square = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StrokeJoin {
    Miter = 0,
    Round = 1,
    Bevel = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TileMode {
    Clamp = 0,
    Repeat = 1,
    Mirror = 2,
}

impl From<TileMode> for skia_safe::TileMode {
    fn from(tile_mode: TileMode) -> Self {
        match tile_mode {
            TileMode::Clamp => skia_safe::TileMode::Clamp,
            TileMode::Mirror => skia_safe::TileMode::Mirror,
            TileMode::Repeat => skia_safe::TileMode::Repeat,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlendMode {
    Clear = 0,
    SourceOver = 1,
    DestinationOver = 2,
    SourceIn = 3,
    DestinationIn = 4,
    SourceOut = 5,
    DestinationOut = 6,
    SourceAtop = 7,
    Xor = 8,
    Multiply = 9,
    Screen = 10,
    Darken = 11,
    Lighten = 12,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FilterQuality {
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

impl From<FilterQuality> for skia_safe::FilterQuality {
    fn from(filter_quality: FilterQuality) -> Self {
        match filter_quality {
            FilterQuality::None => skia_safe::FilterQuality::None,
            FilterQuality::Low => skia_safe::FilterQuality::Low,
            FilterQuality::Medium => skia_safe::FilterQuality::Medium,
            FilterQuality::High => skia_safe::FilterQuality::High,
        }
    }
}

pub struct Surface {
    surface: RefCell<skia_safe::Surface>,
    canvas: Canvas,
}

impl Surface {
    fn new_rgba_with_alpha(width: u32, height: u32, alpha_type: AlphaType) -> Option<Surface> {
        let size = ISize::new(width as i32, height as i32);
        let image_info = ImageInfo::new(size, ColorType::RGBA8888, alpha_type, None);
        let mut surface = skia_safe::Surface::new_raster(&image_info, None, None)?;
        let canvas = Canvas(&mut *surface.canvas());
        Some(Self { surface: RefCell::new(surface), canvas })
    }

    pub fn new_rgba(width: u32, height: u32) -> Option<Surface> {
        Self::new_rgba_with_alpha(width, height, AlphaType::Unpremul)
    }

    pub fn new_rgba_premultiplied(width: u32, height: u32) -> Option<Surface> {
        Self::new_rgba_with_alpha(width, height, AlphaType::Premul)
    }

    pub unsafe fn from_ptr(ptr: *mut skiac_surface) -> Option<Surface> {
        if !ptr.is_null() {
            let mut surface: skia_safe::Surface = std::mem::transmute(ptr);
            let canvas = Canvas(&mut *surface.canvas());
            Some(Self { surface: RefCell::new(surface), canvas })
        }
        else { None }
    }

    pub fn copy_rgba(&self, x: u32, y: u32, width: u32, height: u32) -> Option<Surface> {
        let copy = Surface::new_rgba(width, height)?;
        let point = skia_safe::IPoint::new(-(x as i32), -(y as i32));

        let mut surface = self.surface.borrow_mut();
        let pixmap = surface.peek_pixels()?;
        copy.surface.borrow_mut().write_pixels_from_pixmap(&pixmap, point);

        Some(copy)
    }

    pub fn try_clone(&self) -> Option<Surface> {
        self.copy_rgba(0, 0, self.width(), self.height())
    }

    fn save_png_impl(&self, path: &str) -> Option<()> {
        use std::{ fs::File, io::Write };
        let image = self.surface.borrow_mut().image_snapshot();
        let data = image.encode_to_data(skia_safe::EncodedImageFormat::PNG)?;
        let mut file = File::create(path).ok()?;
        file.write_all(data.as_bytes()).ok()?;
        Some(())
    }

    pub fn save_png(&self, path: &str) -> bool {
        self.save_png_impl(path).is_some()
    }

    pub fn width(&self) -> u32 {
        self.surface.borrow().width() as u32
    }

    pub fn height(&self) -> u32 {
        self.surface.borrow().height() as u32
    }

    pub fn data(&self) -> SurfaceData {
        unsafe {
            let mut surface = self.surface.borrow_mut();
            if let Some(pixels) = surface.peek_pixels() {
                let size = pixels.row_bytes() * pixels.height() as usize;
                SurfaceData { slice: std::slice::from_raw_parts_mut(pixels.addr() as *mut u8, size) }    
            }
            else {
                SurfaceData { slice: std::slice::from_raw_parts_mut(std::ptr::null_mut(), 0) }    
            }
        }
    }

    pub fn data_mut(&mut self) -> SurfaceData {
        if let Some(pixels) = self.surface.borrow_mut().peek_pixels() {
            let size = pixels.row_bytes() * pixels.height() as usize;
            SurfaceData { slice: unsafe { std::slice::from_raw_parts_mut(pixels.addr() as *mut u8, size) } }    
        }
        else {
            SurfaceData { slice: unsafe { std::slice::from_raw_parts_mut(std::ptr::null_mut(), 0) } }
        }
    }

    pub fn is_bgra() -> bool {
        skia_safe::ColorType::n32() == skia_safe::ColorType::RGBA8888
    }
}

impl std::ops::Deref for Surface {
    type Target = Canvas;

    fn deref(&self) -> &Self::Target {
        &self.canvas
    }
}

impl std::ops::DerefMut for Surface {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.canvas
    }
}

pub struct SurfaceData<'a> {
    slice: &'a mut [u8],
}

impl<'a> Deref for SurfaceData<'a> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> DerefMut for SurfaceData<'a> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.slice
    }
}

pub struct Color(u8, u8, u8, u8);

impl Color {
    pub fn new(a: u8, r: u8, g: u8, b: u8) -> Color {
        Color(a, r, g, b)
    }

    pub fn to_u32(&self) -> u32 {
        (self.0 as u32) << 24 | (self.1 as u32) << 16 | (self.2 as u32) << 8 | (self.3 as u32)
    }
}


pub struct Matrix(skia_safe::Matrix);

impl Matrix {
    pub fn new() -> Matrix {
        Self(skia_safe::Matrix::default())
    }

    pub fn new_from(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Matrix {
        Self(skia_safe::Matrix::new_all(
            a as f32,
            c as f32,
            e as f32,
            b as f32,
            d as f32,
            f as f32,
            0.0,
            0.0,
            1.0,
        ))
    }

    pub fn invert(&self) -> Option<Matrix> {
        let matrix = self.0.invert()?;
        Some(Self(matrix))
    }

    pub fn data(&self) -> (f64, f64, f64, f64, f64, f64) {
        let mut d = [0f32; 9];
        self.0.get_9(&mut d);
        (
            d[0] as f64, d[1] as f64, d[2] as f64,
            d[3] as f64, d[4] as f64, d[5] as f64,
        )
    }
}

impl Default for Matrix {
    fn default() -> Matrix {
        Self(skia_safe::Matrix::default())
    }
}

pub struct Canvas(*mut skia_safe::Canvas);

impl Canvas {
    pub unsafe fn from_ptr(ptr: *mut skiac_canvas) -> Option<Canvas> {
        if !ptr.is_null() {
            let canvas: *mut skia_safe::Canvas = std::mem::transmute(ptr);
            Some(Canvas(&mut *canvas))
        }
        else { None }
    }

    fn canvas(&self) -> &mut skia_safe::Canvas {
        unsafe { std::mem::transmute(self.0) }
    }

    pub fn clear(&mut self) {
        self.canvas().clear(skia_safe::Color::TRANSPARENT);
    }

    pub fn fill(&mut self, r: u8, g: u8, b: u8, a: u8) {
        let color = skia_safe::Color::from_argb(a, r, g, b);
        self.canvas().draw_color(color, skia_safe::BlendMode::Src);
    }

    pub fn flush(&mut self) {
        self.canvas().flush();
    }

    pub fn set_matrix(&mut self, matrix: &Matrix) {
        self.canvas().set_matrix(&matrix.0);
    }

    pub fn concat(&mut self, matrix: &Matrix) {
        self.canvas().concat(&matrix.0);
    }

    pub fn scale(&mut self, sx: f64, sy: f64) {
        self.canvas().scale((sx as f32, sy as f32));
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.canvas().translate((dx as f32, dy as f32));
    }

    pub fn get_matrix(&self) -> Matrix {
        Matrix(self.canvas().total_matrix().clone())
    }

    pub fn draw_path(&mut self, path: &Path, paint: &Paint) {
        self.canvas().draw_path(&path.0, &paint.0);
    }

    pub fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64, paint: &Paint) {
        self.canvas().draw_rect(skia_safe::Rect::from_xywh(x as f32, y as f32, w as f32, h as f32), &paint.0);
    }

    pub fn draw_surface(&mut self, surface: &Surface, left: f64, top: f64, alpha: u8,
                        blend_mode: BlendMode, filter_quality: FilterQuality) {
        let image = surface.surface.borrow_mut().image_snapshot();

        let mut paint = Paint::new();
        paint.0.set_filter_quality(filter_quality.into());
        paint.set_alpha(alpha);
        paint.set_blend_mode(blend_mode);
        self.canvas().draw_image(&image, skia_safe::Point::new(left as f32, top as f32), Some(&paint.0));
    }

    pub fn draw_surface_rect(&mut self, surface: &Surface, x: f64, y: f64, w: f64, h: f64,
                             filter_quality: FilterQuality) {
        let image = surface.surface.borrow_mut().image_snapshot();

        let mut paint = Paint::new();
        paint.0.set_filter_quality(filter_quality.into());
        let dst = skia_safe::Rect::from_xywh(x as f32, y as f32, w as f32, h as f32);
        self.canvas().draw_image_rect(&image, None, &dst, &paint.0);
    }

    pub fn reset_matrix(&mut self) {
        self.canvas().reset_matrix();
    }

    pub fn set_clip_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let rect = skia_safe::Rect::from_xywh(x as f32, y as f32, w as f32, h as f32);
        self.canvas().clip_rect(rect, None, true);
    }

    pub fn save(&mut self) {
        self.canvas().save();
    }

    pub fn restore(&mut self) {
        self.canvas().restore();
    }
}

pub struct Paint(skia_safe::Paint);

impl Paint {
    pub fn new() -> Paint {
        Self(skia_safe::Paint::default())
    }
    pub fn set_style(&mut self, style: PaintStyle) {
        self.0.set_style(match style {
            PaintStyle::Fill => skia_safe::PaintStyle::Fill,
            PaintStyle::Stroke => skia_safe::PaintStyle::Stroke,
        });
    }
    pub fn set_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.0.set_argb(a, r, g, b);
    }
    pub fn set_alpha(&mut self, a: u8) {
        self.0.set_alpha(a);
    }
    pub fn set_anti_alias(&mut self, aa: bool) {
        self.0.set_anti_alias(aa);
    }
    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        self.0.set_blend_mode(match blend_mode {
            BlendMode::Clear => skia_safe::BlendMode::Clear,
            BlendMode::SourceOver => skia_safe::BlendMode::SrcOver,
            BlendMode::DestinationOver => skia_safe::BlendMode::DstOver,
            BlendMode::SourceIn => skia_safe::BlendMode::SrcIn,
            BlendMode::DestinationIn => skia_safe::BlendMode::DstIn,
            BlendMode::SourceOut => skia_safe::BlendMode::SrcOut,
            BlendMode::DestinationOut => skia_safe::BlendMode::DstOut,
            BlendMode::SourceAtop => skia_safe::BlendMode::SrcATop,
            BlendMode::Xor => skia_safe::BlendMode::Xor,
            BlendMode::Multiply => skia_safe::BlendMode::Multiply,
            BlendMode::Screen => skia_safe::BlendMode::Screen,
            BlendMode::Darken => skia_safe::BlendMode::Darken,
            BlendMode::Lighten => skia_safe::BlendMode::Lighten,
        });
    }
    pub fn set_shader(&mut self, shader: &Shader) {
        self.0.set_shader(Some(&shader.0));
    }
    pub fn set_stroke_width(&mut self, width: f64) {
        self.0.set_stroke_width(width as f32);
    }
    pub fn set_stroke_cap(&mut self, cap: StrokeCap) {
        self.0.set_stroke_cap(match cap {
            StrokeCap::Butt => skia_safe::paint::Cap::Butt,
            StrokeCap::Round => skia_safe::paint::Cap::Round,
            StrokeCap::Square => skia_safe::paint::Cap::Square,
        });
    }
    pub fn set_stroke_join(&mut self, join: StrokeJoin) {
        self.0.set_stroke_join(match join {
            StrokeJoin::Bevel => skia_safe::paint::Join::Bevel,
            StrokeJoin::Miter => skia_safe::paint::Join::Miter,
            StrokeJoin::Round => skia_safe::paint::Join::Round,
        });
    }
    pub fn set_stroke_miter(&mut self, miter: f64) {
        self.0.set_stroke_miter(miter as f32);
    }
    pub fn set_path_effect(&mut self, path_effect: PathEffect) {
        self.0.set_path_effect(Some(&path_effect.0));
    }
}

pub struct Path(skia_safe::Path);

impl Path {
    pub fn new() -> Path {
        Self(skia_safe::Path::default())
    }

    pub fn set_fill_type(&mut self, kind: FillType) {
        self.0.set_fill_type(match kind {
            FillType::EvenOdd => skia_safe::path::FillType::EvenOdd,
            FillType::Winding => skia_safe::path::FillType::Winding,
        });
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.0.move_to(skia_safe::Point::new(x as f32, y as f32));
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        self.0.line_to(skia_safe::Point::new(x as f32, y as f32));
    }

    pub fn cubic_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
        self.0.cubic_to(
            skia_safe::Point::new(x1 as f32, y1 as f32),
            skia_safe::Point::new(x2 as f32, y2 as f32),
            skia_safe::Point::new(x3 as f32, y3 as f32),
        );
    }

    pub fn close(&mut self) {
        self.0.close();
    }
}

pub struct Gradient {
    pub colors: Vec<u32>,
    pub positions: Vec<f32>,
    pub tile_mode: TileMode,
    pub matrix: Matrix
}

pub struct LinearGradient {
    pub start_point: (f64, f64),
    pub end_point: (f64, f64),
    pub base: Gradient
}

pub struct RadialGradient {
    pub start_circle: (f64, f64, f64),
    pub end_circle: (f64, f64, f64),
    pub base: Gradient
}

pub struct Shader(skia_safe::Shader);

impl Shader {
    pub fn new_linear_gradient(grad:  LinearGradient) -> Shader {
        let start_point = skia_safe::Point::new(grad.start_point.0 as f32, grad.start_point.1 as f32);
        let end_point = skia_safe::Point::new(grad.end_point.0 as f32, grad.end_point.1 as f32);

        let colors: Vec<skia_safe::Color> = grad.base.colors.iter().map(|x| (*x).into()).collect();

        let shader = skia_safe::Shader::linear_gradient(
            (start_point, end_point),
            colors.as_slice(),
            Some(grad.base.positions.as_slice()),
            grad.base.tile_mode.into(),
            None,
            Some(&grad.base.matrix.0),
        );

        Self(shader.unwrap())
    }

    pub fn new_radial_gradient(grad: RadialGradient) -> Shader {
        let start_point = skia_safe::Point::new(grad.start_circle.0 as f32, grad.start_circle.1 as f32);
        let end_point = skia_safe::Point::new(grad.start_circle.0 as f32, grad.start_circle.1 as f32);

        let start_radius = grad.start_circle.2 as f32;
        let end_radius = grad.end_circle.2 as f32;

        let colors: Vec<skia_safe::Color> = grad.base.colors.iter().map(|x| (*x).into()).collect();

        let shader = skia_safe::Shader::two_point_conical_gradient(
            start_point, start_radius,
            end_point, end_radius,
            colors.as_slice(),
            Some(grad.base.positions.as_slice()),
            grad.base.tile_mode.into(),
            None,
            Some(&grad.base.matrix.0),
        );

        Self(shader.unwrap())
    }

    pub fn new_from_surface_image(surface: &Surface, matrix: Matrix) -> Shader {
        let image = surface.surface.borrow_mut().image_snapshot();
        let tile_mode = skia_safe::TileMode::Repeat;
        let tile_modes = Some((tile_mode, tile_mode));
        Self(image.to_shader(tile_modes, Some(&matrix.0)))
    }
}

pub struct PathEffect(skia_safe::PathEffect);

impl PathEffect {
    pub fn new_dash_path(intervals: &[f32], phase: f32) -> PathEffect {
        let path_effect = skia_safe::dash_path_effect::new(&intervals, phase);
        Self(path_effect.unwrap())
    }
}
