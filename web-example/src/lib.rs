use cotis::cotis_app::{AsyncRenderApp, CotisApp};
use cotis::element_configuring::ConfiguredParentElement;
use cotis::utils::ElementIdConfig;
use cotis_defaults::colors::Color;
#[cfg(any(feature = "complex_color", feature = "complex_colored_text"))]
use cotis_defaults::colors::ColorLayer;
#[cfg(any(feature = "complex_color", feature = "complex_colored_text"))]
use cotis_defaults::colors::{ColorAttachmentPoint, ColorPos, GradientStop, LinearGradient};
use cotis_defaults::element_configs::ElementConfig;
use cotis_defaults::element_configs::style::sizing::{AxisSizing, DoubleAxisSizing, Sizing};
use cotis_defaults::element_configs::style::types::{
    Alignment, LayoutAlignmentX, LayoutAlignmentY, LayoutDirection, Padding,
};
use cotis_defaults::element_configs::style::{
    BorderElementStyle, BorderWidthStyle, CornerRadiiStyle, CotisStyle, LayoutStyle,
};
use cotis_defaults::element_configs::text_config::{
    TextAlignment, TextConfig, TextElementConfigWrapMode,
};
use cotis_defaults::element_configs::text_config::{TextElementConfig, TextStyle};
use cotis_defaults::render_commands::render_t_list::RenderTList;
use cotis_defaults::render_commands::{Border, ClipEnd, ClipStart, Image, Rectangle, Text};
use cotis_layout::layout_struct::RenderCommandOutput;
use cotis_layout::preamble::{CotisLayoutManager, CotisLayoutRun};
use cotis_macros::cotis_start_async;
use cotis_pipes::cotis_layout_pipes::CotisLayoutToRenderListPipeForGenerics;
use cotis_utils::math::Dimensions;
use cotis_web::custom_data::ExtraHTMLData;
use cotis_web::images::URLImageEnum;
use cotis_web::renderer::HTMLRenderer;
use log::info;
use wasm_bindgen::prelude::*;

#[cotis_start_async]
async fn start() {
    info!("secondary main V1");
    #[cfg(feature = "complex_colored_text")]
    info!("complex_colored_text");
    #[cfg(feature = "complex_color")]
    info!("complex_color");
    entry_async(HTMLRenderer::new()).await;
}

#[cfg(not(feature = "complex_color"))]
fn style_bg(color: Color) -> Color {
    color
}

#[cfg(feature = "complex_color")]
fn style_bg(color: Color) -> ColorLayer {
    ColorLayer::Solid(color)
}

#[cfg(not(feature = "complex_colored_text"))]
fn text_fg(color: Color) -> Color {
    color
}

#[cfg(feature = "complex_colored_text")]
fn text_fg(color: Color) -> ColorLayer {
    ColorLayer::Solid(color)
}

#[cfg(not(feature = "complex_color"))]
fn demo_bg_gradient() -> Color {
    Color::rgba(255.0, 255.0, 255.0, 255.0)
}

#[cfg(feature = "complex_color")]
fn demo_bg_gradient() -> ColorLayer {
    ColorLayer::Linear(LinearGradient {
        start: ColorPos {
            x: 0.0,
            y: 0.0,
            attachment_point: ColorAttachmentPoint::TopLeft,
        },
        end: ColorPos {
            x: 1.0,
            y: 1.0,
            attachment_point: ColorAttachmentPoint::BottomRight,
        },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color::rgba(255.0, 255.0, 255.0, 255.0),
            },
            GradientStop {
                offset: 1.0,
                color: Color::rgba(242.0, 247.0, 255.0, 255.0),
            },
        ],
    })
}

#[cfg(not(feature = "complex_colored_text"))]
fn demo_text_gradient() -> Color {
    Color::rgba(252.0, 63.0, 78.0, 255.0)
}

#[cfg(feature = "complex_colored_text")]
fn demo_text_gradient() -> ColorLayer {
    ColorLayer::Linear(LinearGradient {
        start: ColorPos {
            x: 0.0,
            y: 0.0,
            attachment_point: ColorAttachmentPoint::TopLeft,
        },
        end: ColorPos {
            x: 1.0,
            y: 0.0,
            attachment_point: ColorAttachmentPoint::TopRight,
        },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color::rgba(252.0, 63.0, 78.0, 255.0),
            },
            GradientStop {
                offset: 1.0,
                color: Color::rgba(110.0, 95.0, 255.0, 255.0),
            },
        ],
    })
}

type HTMLElementConfig = ElementConfig<'static, Sizing, URLImageEnum, (), ExtraHTMLData>;

pub type StandardRenderCmds = RenderTList<
    ClipStart<'static, ExtraHTMLData>,
    RenderTList<
        ClipEnd,
        RenderTList<
            Rectangle<'static, ExtraHTMLData>,
            RenderTList<
                Image<'static, URLImageEnum, ExtraHTMLData>,
                RenderTList<
                    Border<'static, ExtraHTMLData>,
                    RenderTList<Text<'static, ExtraHTMLData>, ()>,
                >,
            >,
        >,
    >,
>;

async fn entry_async(render: HTMLRenderer) {
    let manager = CotisLayoutManager::new(Dimensions::new(800.0, 800.0));
    let mut manager: CotisApp<_, _, _> =
        CotisApp::new(render, manager, CotisLayoutToRenderListPipeForGenerics);
    loop {
        AsyncRenderApp::<
            HTMLElementConfig,
            RenderCommandOutput<HTMLElementConfig>,
            StandardRenderCmds,
            _,
            _,
        >::compute_frame_async(
            &mut manager,
            |layout: &mut ConfiguredParentElement<
                '_,
                CotisLayoutRun<'_, HTMLElementConfig>,
                HTMLElementConfig,
            >| {
                draw_as_child(layout);
            },
        )
        .await;
    }
}

pub fn draw_as_child<'a, 'b, Conf>(
    parent: &mut cotis::element_configuring::ConfiguredParentElement<'b, Conf, HTMLElementConfig>,
) where
    Conf: cotis::element_configuring::ConfigureElements<HTMLElementConfig>
        + cotis::element_configuring::ConfigureLeafElements<TextElementConfig<'a>>,
{
    let mut el0 = parent.new_element();
    el0.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 32.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el0_p = el0.make_parent();
    let mut el1 = el0_p.new_element();
    el1.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 30.4,
                    bottom: 25.6,
                    right: 32.0,
                    left: 32.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: demo_bg_gradient(),
            corner_radius: CornerRadiiStyle {
                top_left: 16.0,
                top_right: 16.0,
                bottom_left: 16.0,
                bottom_right: 16.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(255.0, 255.0, 255.0, 102.0),
                width: BorderWidthStyle {
                    left: 1.0,
                    right: 1.0,
                    top: 2.0,
                    bottom: 1.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el1_p = el1.make_parent();
    let mut el2 = el1_p.new_element();
    el2.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el2_p = el2.make_parent();
    let mut el3 = el2_p.new_element();
    el3.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el3_p = el3.make_parent();
    let mut el4 = el3_p.new_element();
    el4.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::LeftToRight,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el4_p = el4.make_parent();
    let mut el5 = el4_p.new_element();
    el5.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    el5.set_leaf_config(TextElementConfig {
        text: cotis::utils::OwnedOrRef::Owned("Underrun – Making Of".to_owned().into_boxed_str()),
        config: TextConfig {
            font_id: 0,
            font_size: 48.0,
            letter_spacing: -1.28,
            line_height: 1.05,
            wrap_mode: TextElementConfigWrapMode::Words,
            alignment: TextAlignment::Left,
        },
        style: TextStyle {
            color: text_fg(Color::rgba(0.0, 0.0, 0.0, 255.0)),
        },
    });
    el4_p.close_element();
    el3_p.close_element();
    el2_p.close_element();
    let mut el6 = el1_p.new_element();
    el6.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fixed(32.0),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::LeftToRight,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    el6.close_element();
    let mut el7 = el1_p.new_element();
    el7.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el7_p = el7.make_parent();
    let mut el8 = el7_p.new_element();
    el8.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el8_p = el8.make_parent();
    let mut el9 = el8_p.new_element();
    el9.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el9_p = el9.make_parent();
    let mut el10 = el9_p.new_element();
    el10.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    el10.set_leaf_config(TextElementConfig {
        text: cotis::utils::OwnedOrRef::Owned(
            "— Thursday, September 20th 2018"
                .to_owned()
                .into_boxed_str(),
        ),
        config: TextConfig {
            font_id: 0,
            font_size: 14.4,
            letter_spacing: 0.0,
            line_height: 0.0,
            wrap_mode: TextElementConfigWrapMode::Words,
            alignment: TextAlignment::Left,
        },
        style: TextStyle {
            color: text_fg(Color::rgba(0.0, 0.0, 0.0, 255.0)),
        },
    });
    el9_p.close_element();
    el8_p.close_element();
    el7_p.close_element();
    let mut el11 = el1_p.new_element();
    el11.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fixed(32.0),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::LeftToRight,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    el11.close_element();
    let mut el12 = el1_p.new_element();
    el12.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el12_p = el12.make_parent();
    let mut el13 = el12_p.new_element();
    el13.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el13_p = el13.make_parent();
    let mut el14 = el13_p.new_element();
    el14.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    el14.set_leaf_config(TextElementConfig {
        text: cotis::utils::OwnedOrRef::Owned(
            "How I compressed a twin-stick WebGL shooter game into 13kb of JavaScript"
                .to_owned()
                .into_boxed_str(),
        ),
        config: TextConfig {
            font_id: 0,
            font_size: 16.0,
            letter_spacing: 0.0,
            line_height: 0.0,
            wrap_mode: TextElementConfigWrapMode::Words,
            alignment: TextAlignment::Left,
        },
        style: TextStyle {
            color: text_fg(Color::rgba(0.0, 0.0, 0.0, 255.0)),
        },
    });
    el13_p.close_element();
    el12_p.close_element();
    let mut el15 = el1_p.new_element();
    el15.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fixed(24.0),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::LeftToRight,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    el15.close_element();
    let mut el16 = el1_p.new_element();
    el16.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el16_p = el16.make_parent();
    let mut el17 = el16_p.new_element();
    el17.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::TopToBottom,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el17_p = el17.make_parent();
    let mut el18 = el17_p.new_element();
    el18.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            layout: LayoutStyle {
                padding: Padding {
                    top: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    left: 0.0,
                },
                child_gap: 0.0,
                child_alignment: Alignment {
                    x: LayoutAlignmentX::Left,
                    y: LayoutAlignmentY::Top,
                },
                layout_direction: LayoutDirection::LeftToRight,
            },
            background_color: style_bg(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            corner_radius: CornerRadiiStyle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
            floating: None,
            border: BorderElementStyle {
                color: Color::rgba(0.0, 0.0, 0.0, 255.0),
                width: BorderWidthStyle {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                    between_children: 0.0,
                },
            },
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    let mut el18_p = el18.make_parent();
    let mut el19 = el18_p.new_element();
    el19.set_config(ElementConfig {
        id_config: ElementIdConfig::new_empty(),
        style: CotisStyle {
            sizing: Sizing::DoubleAxis(DoubleAxisSizing {
                width: AxisSizing::Fit(0.0, f32::MAX),
                height: AxisSizing::Fit(0.0, f32::MAX),
            }),
            ..Default::default()
        },
        image: None,
        custom: None,
        extra_data: None,
    });
    el19.set_leaf_config(TextElementConfig {
        text: cotis::utils::OwnedOrRef::Owned("Read ›".to_owned().into_boxed_str()),
        config: TextConfig {
            font_id: 0,
            font_size: 16.0,
            letter_spacing: 0.0,
            line_height: 0.0,
            wrap_mode: TextElementConfigWrapMode::Words,
            alignment: TextAlignment::Left,
        },
        style: TextStyle {
            color: demo_text_gradient(),
        },
    });
    el18_p.close_element();
    el17_p.close_element();
    el16_p.close_element();
    el1_p.close_element();
    el0_p.close_element();
}
