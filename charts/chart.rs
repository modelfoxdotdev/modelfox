use crate::config::{ChartColors, ChartConfig, DARK_CHART_COLORS, LIGHT_CHART_COLORS};
use num::ToPrimitive;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys as dom;

pub struct Chart<T>
where
	T: ChartImpl,
{
	active_hover_regions: Vec<ActiveHoverRegion<T::HoverRegionInfo>>,
	chart_canvas: dom::HtmlCanvasElement,
	chart_colors: Option<ChartColors>,
	chart_config: Option<ChartConfig>,
	color_scheme_media_query: Option<dom::MediaQueryList>,
	container: dom::HtmlDivElement,
	hover_regions: Option<Vec<HoverRegion<T::HoverRegionInfo>>>,
	intersection_callback: Option<Closure<dyn Fn(JsValue, dom::IntersectionObserver)>>,
	intersection_observer: Option<dom::IntersectionObserver>,
	on_color_scheme_media_query_change: Option<Closure<dyn Fn()>>,
	on_mouse_event: Option<Closure<dyn Fn(dom::MouseEvent)>>,
	on_resize: Option<Closure<dyn Fn()>>,
	on_touch_event: Option<Closure<dyn Fn(dom::TouchEvent)>>,
	options: T::Options,
	overlay_canvas: dom::HtmlCanvasElement,
	overlay_div: dom::HtmlElement,
	overlay_info: Option<T::OverlayInfo>,
}

pub trait ChartImpl: 'static {
	type Options;
	type OverlayInfo;
	type HoverRegionInfo: Clone;
	fn draw_chart(
		options: DrawChartOptions<Self::Options>,
	) -> DrawChartOutput<Self::OverlayInfo, Self::HoverRegionInfo>;
	fn draw_overlay(
		options: DrawOverlayOptions<Self::Options, Self::OverlayInfo, Self::HoverRegionInfo>,
	);
}

pub struct DrawChartOptions<'a, Options> {
	pub chart_colors: &'a ChartColors,
	pub chart_config: &'a ChartConfig,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub options: &'a Options,
}

pub struct DrawChartOutput<OverlayInfo, HoverRegionInfo>
where
	HoverRegionInfo: Clone,
{
	pub hover_regions: Vec<HoverRegion<HoverRegionInfo>>,
	pub overlay_info: OverlayInfo,
}

pub struct DrawOverlayOptions<'a, Options, OverlayInfo, HoverRegionInfo>
where
	HoverRegionInfo: Clone,
{
	pub active_hover_regions: &'a [ActiveHoverRegion<HoverRegionInfo>],
	pub chart_colors: &'a ChartColors,
	pub chart_config: &'a ChartConfig,
	pub ctx: &'a dom::CanvasRenderingContext2d,
	pub options: &'a Options,
	pub overlay_info: &'a OverlayInfo,
	pub overlay_div: &'a dom::HtmlElement,
}

pub struct HoverRegion<HoverRegionInfo>
where
	HoverRegionInfo: Clone,
{
	pub distance: Box<dyn Fn(f64, f64) -> f64>,
	pub hit_test: Box<dyn Fn(f64, f64) -> bool>,
	pub info: HoverRegionInfo,
}

#[derive(Clone)]
pub struct ActiveHoverRegion<HoverRegionInfo>
where
	HoverRegionInfo: Clone,
{
	pub distance: f64,
	pub info: HoverRegionInfo,
}

impl<T> Chart<T>
where
	T: ChartImpl,
{
	pub fn new(container: dom::HtmlDivElement, options: T::Options) -> Rc<RefCell<Chart<T>>> {
		// Create the chart canvas, overlay div (for tooltips), and overlay canvas (for crosshairs).
		let window = dom::window().unwrap();
		let document = window.document().unwrap();
		container
			.style()
			.set_property("position", "relative")
			.unwrap();
		let chart_canvas = document
			.create_element("canvas")
			.unwrap()
			.dyn_into::<dom::HtmlCanvasElement>()
			.unwrap();
		chart_canvas.style().set_css_text(
			r#"
				position: absolute;
				top: 0;
				bottom: 0;
				left: 0;
				right: 0;
			"#,
		);
		container.append_child(&chart_canvas).unwrap();
		let overlay_div = document
			.create_element("div")
			.unwrap()
			.dyn_into::<dom::HtmlElement>()
			.unwrap();
		overlay_div.style().set_css_text(
			r#"
				position: absolute;
				top: 0;
				bottom: 0;
				left: 0;
				right: 0;
			"#,
		);
		container.append_child(&overlay_div).unwrap();
		let overlay_canvas = document
			.create_element("canvas")
			.unwrap()
			.dyn_into::<dom::HtmlCanvasElement>()
			.unwrap();
		overlay_canvas.style().set_css_text(
			r#"
				position: absolute;
				top: 0;
				bottom: 0;
				left: 0;
				right: 0;
			"#,
		);
		container.append_child(&overlay_canvas).unwrap();
		// Create the Chart.
		let chart = Rc::new(RefCell::new(Chart {
			active_hover_regions: Vec::new(),
			chart_canvas,
			chart_colors: None,
			chart_config: Some(ChartConfig::default()),
			color_scheme_media_query: None,
			container,
			hover_regions: None,
			intersection_callback: None,
			intersection_observer: None,
			on_color_scheme_media_query_change: None,
			on_mouse_event: None,
			on_resize: None,
			on_touch_event: None,
			options,
			overlay_canvas,
			overlay_div,
			overlay_info: None,
		}));
		// Add the intersection observer.
		let chart_ref = Rc::downgrade(&chart);
		let intersection_callback = Closure::<dyn Fn(_, _)>::wrap(Box::new(move |_, _| {
			let chart = chart_ref.upgrade().unwrap();
			let mut chart = chart.borrow_mut();
			chart.draw_chart();
			chart.draw_overlay();
			chart.intersection_observer.take().unwrap().disconnect();
			chart.intersection_callback.take().unwrap();
		}));
		let intersection_observer =
			dom::IntersectionObserver::new(intersection_callback.as_ref().unchecked_ref()).unwrap();
		intersection_observer.observe(&chart.borrow().container);
		chart.borrow_mut().intersection_callback = Some(intersection_callback);
		chart.borrow_mut().intersection_observer = Some(intersection_observer);
		// Add the mouse move handler.
		let chart_ref = Rc::downgrade(&chart);
		let on_mouse_event = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: dom::MouseEvent| {
			let chart = chart_ref.upgrade().unwrap();
			let mut chart = chart.borrow_mut();
			let canvas_client_rect = chart.chart_canvas.get_bounding_client_rect();
			let x = event.client_x().to_f64().unwrap() - canvas_client_rect.left();
			let y = event.client_y().to_f64().unwrap() - canvas_client_rect.top();
			chart.update_active_hover_regions(x, y);
			chart.draw_overlay();
		}));
		chart
			.borrow_mut()
			.overlay_canvas
			.add_event_listener_with_callback("mouseenter", on_mouse_event.as_ref().unchecked_ref())
			.unwrap();
		chart
			.borrow_mut()
			.overlay_canvas
			.add_event_listener_with_callback("mouseleave", on_mouse_event.as_ref().unchecked_ref())
			.unwrap();
		chart
			.borrow_mut()
			.overlay_canvas
			.add_event_listener_with_callback("mousemove", on_mouse_event.as_ref().unchecked_ref())
			.unwrap();
		chart.borrow_mut().on_mouse_event = Some(on_mouse_event);
		// Add the touch event handler.
		let chart_ref = Rc::downgrade(&chart);
		let on_touch_event = Closure::<dyn Fn(_)>::wrap(Box::new(move |event: dom::TouchEvent| {
			let chart = chart_ref.upgrade().unwrap();
			let mut chart = chart.borrow_mut();
			let canvas_client_rect = chart.chart_canvas.get_bounding_client_rect();
			let touch = event.touches().get(0).unwrap();
			let x = touch.client_x().to_f64().unwrap() - canvas_client_rect.left();
			let y = touch.client_y().to_f64().unwrap() - canvas_client_rect.top();
			chart.update_active_hover_regions(x, y);
			chart.draw_overlay();
		}));
		chart
			.borrow_mut()
			.chart_canvas
			.add_event_listener_with_callback("touchstart", on_touch_event.as_ref().unchecked_ref())
			.unwrap();
		chart.borrow_mut().on_touch_event = Some(on_touch_event);
		// Add the resize handler.
		let chart_ref = Rc::downgrade(&chart);
		let on_resize = Closure::<dyn Fn()>::wrap(Box::new(move || {
			let chart = chart_ref.upgrade().unwrap();
			let mut chart = chart.borrow_mut();
			chart.draw_chart();
			chart.draw_overlay();
		}));
		window
			.add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())
			.unwrap();
		chart.borrow_mut().on_resize = Some(on_resize);
		// Add the color scheme handler.
		let color_scheme_media_query = window
			.match_media("(prefers-color-scheme: dark)")
			.unwrap()
			.unwrap();
		let chart_ref = Rc::downgrade(&chart);
		let on_color_scheme_media_query_change = Closure::<dyn Fn()>::wrap(Box::new(move || {
			let chart = chart_ref.upgrade().unwrap();
			let mut chart = chart.borrow_mut();
			chart.draw_chart();
			chart.draw_overlay();
		}));
		color_scheme_media_query
			.add_event_listener_with_callback(
				"change",
				on_color_scheme_media_query_change.as_ref().unchecked_ref(),
			)
			.unwrap();
		chart.borrow_mut().color_scheme_media_query = Some(color_scheme_media_query);
		chart.borrow_mut().on_color_scheme_media_query_change =
			Some(on_color_scheme_media_query_change);
		chart
	}

	pub fn update(&mut self, options: T::Options) {
		self.options = options;
		self.draw_chart();
		self.draw_overlay();
	}

	fn draw_chart(&mut self) {
		let width = self.container.client_width().to_f64().unwrap();
		let height = self.container.client_height().to_f64().unwrap();
		let dpr = dom::window().unwrap().device_pixel_ratio();
		self.chart_canvas.set_width((width * dpr).to_u32().unwrap());
		self.chart_canvas
			.set_height((height * dpr).to_u32().unwrap());
		self.chart_canvas
			.style()
			.set_property("width", &format!("{}px", width))
			.unwrap();
		self.chart_canvas
			.style()
			.set_property("height", &format!("{}px", height))
			.unwrap();
		self.chart_colors = Some(
			if self.color_scheme_media_query.as_ref().unwrap().matches() {
				DARK_CHART_COLORS
			} else {
				LIGHT_CHART_COLORS
			},
		);
		let ctx = self
			.chart_canvas
			.get_context("2d")
			.unwrap()
			.unwrap()
			.dyn_into::<dom::CanvasRenderingContext2d>()
			.unwrap();
		ctx.scale(dpr, dpr).unwrap();
		ctx.clear_rect(0.0, 0.0, width, height);
		ctx.set_font(&self.chart_config.as_ref().unwrap().font);
		// Do not draw the chart if the width or height is 0.
		if width == 0.0 || height == 0.0 {
			self.hover_regions = None;
			self.overlay_info = None;
			return;
		}
		let output = T::draw_chart(DrawChartOptions {
			chart_colors: self.chart_colors.as_ref().unwrap(),
			chart_config: self.chart_config.as_ref().unwrap(),
			ctx: &ctx,
			options: &mut self.options,
		});
		self.hover_regions = Some(output.hover_regions);
		self.overlay_info = Some(output.overlay_info);
	}

	fn draw_overlay(&mut self) {
		let width = self.container.client_width().to_f64().unwrap();
		let height = self.container.client_height().to_f64().unwrap();
		let dpr = dom::window().unwrap().device_pixel_ratio();
		self.overlay_canvas
			.set_width((width * dpr).to_u32().unwrap());
		self.overlay_canvas
			.set_height((height * dpr).to_u32().unwrap());
		self.overlay_canvas
			.style()
			.set_property("width", &format!("{}px", width))
			.unwrap();
		self.overlay_canvas
			.style()
			.set_property("height", &format!("{}px", height))
			.unwrap();
		self.chart_colors = Some(
			if self.color_scheme_media_query.as_ref().unwrap().matches() {
				DARK_CHART_COLORS
			} else {
				LIGHT_CHART_COLORS
			},
		);
		let ctx = self
			.overlay_canvas
			.get_context("2d")
			.unwrap()
			.unwrap()
			.dyn_into::<dom::CanvasRenderingContext2d>()
			.unwrap();
		ctx.scale(dpr, dpr).unwrap();
		ctx.clear_rect(0.0, 0.0, width, height);
		ctx.set_font(&self.chart_config.as_ref().unwrap().font);
		let children = self.overlay_div.child_nodes();
		for i in 0..children.length() {
			let child = &children.get(i).unwrap();
			self.overlay_div.remove_child(child).unwrap();
		}
		// Do not draw the overlay if the width or height is 0.
		if width == 0.0 || height == 0.0 {
			return;
		}
		T::draw_overlay(DrawOverlayOptions {
			active_hover_regions: &self.active_hover_regions,
			chart_colors: &self.chart_colors.as_ref().unwrap(),
			chart_config: &self.chart_config.as_ref().unwrap(),
			ctx: &ctx,
			options: &self.options,
			overlay_info: self.overlay_info.as_ref().unwrap(),
			overlay_div: &self.overlay_div,
		});
	}

	fn update_active_hover_regions(&mut self, x: f64, y: f64) {
		self.active_hover_regions = Vec::new();
		if let Some(hover_regions) = self.hover_regions.as_ref() {
			for hover_region in hover_regions {
				if (hover_region.hit_test)(x, y) {
					self.active_hover_regions.push(ActiveHoverRegion {
						distance: (hover_region.distance)(x, y),
						info: hover_region.info.clone(),
					});
				}
			}
		}
	}
}

impl<T> Drop for Chart<T>
where
	T: ChartImpl,
{
	fn drop(&mut self) {
		// Remove event listeners.
		let on_mouse_event = self.on_mouse_event.as_ref().unwrap();
		self.chart_canvas
			.remove_event_listener_with_callback(
				"mouseenter",
				on_mouse_event.as_ref().unchecked_ref(),
			)
			.unwrap();
		self.chart_canvas
			.remove_event_listener_with_callback(
				"mouseleave",
				on_mouse_event.as_ref().unchecked_ref(),
			)
			.unwrap();
		self.chart_canvas
			.remove_event_listener_with_callback(
				"mousemove",
				on_mouse_event.as_ref().unchecked_ref(),
			)
			.unwrap();
		let on_touch_event = self.on_touch_event.as_ref().unwrap();
		self.chart_canvas
			.remove_event_listener_with_callback(
				"touchstart",
				on_touch_event.as_ref().unchecked_ref(),
			)
			.unwrap();
		let on_resize = self.on_resize.as_ref().unwrap();
		dom::window()
			.unwrap()
			.remove_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())
			.unwrap();
		let on_color_scheme_media_query_change =
			self.on_color_scheme_media_query_change.as_ref().unwrap();
		self.color_scheme_media_query
			.as_ref()
			.unwrap()
			.remove_event_listener_with_callback(
				"change",
				on_color_scheme_media_query_change.as_ref().unchecked_ref(),
			)
			.unwrap();
		// Remove html elements.
		self.container.remove_child(&self.chart_canvas).unwrap();
		self.container.remove_child(&self.overlay_canvas).unwrap();
		self.container.remove_child(&self.overlay_div).unwrap();
	}
}
