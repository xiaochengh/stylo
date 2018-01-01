/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for box properties.

use Atom;
use cssparser::Parser;
use parser::{Parse, ParserContext};
#[cfg(feature = "servo")]
use properties::{longhands, PropertyDeclaration};
use std::fmt;
use style_traits::{ParseError, ToCss};
use values::CustomIdent;
use values::KeyframesName;
#[cfg(feature = "servo")]
use values::computed::Context;
use values::generics::box_::AnimationIterationCount as GenericAnimationIterationCount;
use values::generics::box_::VerticalAlign as GenericVerticalAlign;
use values::specified::{AllowQuirks, Number};
use values::specified::length::LengthOrPercentage;

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq, ToComputedValue, ToCss)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
/// Defines an element’s display type, which consists of
/// the two basic qualities of how an element generates boxes
/// <https://drafts.csswg.org/css-display/#propdef-display>
pub enum Display {
    Inline, Block, InlineBlock,
    Table, InlineTable, TableRowGroup, TableHeaderGroup,
    TableFooterGroup, TableRow, TableColumnGroup,
    TableColumn, TableCell, TableCaption, ListItem, None,
    #[parse(aliases = "-webkit-flex")]
    Flex,
    #[parse(aliases = "-webkit-inline-flex")]
    InlineFlex,
    #[cfg(feature = "gecko")]
    Grid,
    #[cfg(feature = "gecko")]
    InlineGrid,
    #[cfg(feature = "gecko")]
    Ruby,
    #[cfg(feature = "gecko")]
    RubyBase,
    #[cfg(feature = "gecko")]
    RubyBaseContainer,
    #[cfg(feature = "gecko")]
    RubyText,
    #[cfg(feature = "gecko")]
    RubyTextContainer,
    #[cfg(feature = "gecko")]
    Contents,
    #[cfg(feature = "gecko")]
    FlowRoot,
    #[cfg(feature = "gecko")]
    WebkitBox,
    #[cfg(feature = "gecko")]
    WebkitInlineBox,
    #[cfg(feature = "gecko")]
    MozBox,
    #[cfg(feature = "gecko")]
    MozInlineBox,
    #[cfg(feature = "gecko")]
    MozGrid,
    #[cfg(feature = "gecko")]
    MozInlineGrid,
    #[cfg(feature = "gecko")]
    MozGridGroup,
    #[cfg(feature = "gecko")]
    MozGridLine,
    #[cfg(feature = "gecko")]
    MozStack,
    #[cfg(feature = "gecko")]
    MozInlineStack,
    #[cfg(feature = "gecko")]
    MozDeck,
    #[cfg(feature = "gecko")]
    MozPopup,
    #[cfg(feature = "gecko")]
    MozGroupbox,
}

impl Display {
    /// The initial display value.
    #[inline]
    pub fn inline() -> Self {
        Display::Inline
    }

    /// Returns whether this "display" value is the display of a flex or
    /// grid container.
    ///
    /// This is used to implement various style fixups.
    pub fn is_item_container(&self) -> bool {
        match *self {
            Display::Flex | Display::InlineFlex => true,
            #[cfg(feature = "gecko")]
            Display::Grid | Display::InlineGrid => true,
            _ => false,
        }
    }

    /// Returns whether an element with this display type is a line
    /// participant, which means it may lay its children on the same
    /// line as itself.
    pub fn is_line_participant(&self) -> bool {
        match *self {
            Display::Inline => true,
            #[cfg(feature = "gecko")]
            Display::Contents |
            Display::Ruby |
            Display::RubyBaseContainer => true,
            _ => false,
        }
    }

    /// Whether `new_display` should be ignored, given a previous
    /// `old_display` value.
    ///
    /// This is used to ignore `display: -moz-box` declarations after an
    /// equivalent `display: -webkit-box` declaration, since the former
    /// has a vastly different meaning. See bug 1107378 and bug 1407701.
    ///
    /// FIXME(emilio): This is a pretty decent hack, we should try to
    /// remove it.
    pub fn should_ignore_parsed_value(
        _old_display: Self,
        _new_display: Self,
    ) -> bool {
        #[cfg(feature = "gecko")] {
            match (_old_display, _new_display) {
                (Display::WebkitBox, Display::MozBox) |
                (Display::WebkitInlineBox, Display::MozInlineBox) => {
                    return true;
                }
                _ => {},
            }
        }

        return false;
    }

    /// Returns whether this "display" value is one of the types for
    /// ruby.
    #[cfg(feature = "gecko")]
    pub fn is_ruby_type(&self) -> bool {
        matches!(*self,
            Display::Ruby |
            Display::RubyBase |
            Display::RubyText |
            Display::RubyBaseContainer |
            Display::RubyTextContainer
        )
    }

    /// Returns whether this "display" value is a ruby level container.
    #[cfg(feature = "gecko")]
    pub fn is_ruby_level_container(&self) -> bool {
        matches!(*self,
            Display::RubyBaseContainer |
            Display::RubyTextContainer
        )
    }

    /// Convert this display into an equivalent block display.
    ///
    /// Also used for style adjustments.
    pub fn equivalent_block_display(&self, _is_root_element: bool) -> Self {
        match *self {
            // Values that have a corresponding block-outside version.
            Display::InlineTable => Display::Table,
            Display::InlineFlex => Display::Flex,

            #[cfg(feature = "gecko")]
            Display::InlineGrid => Display::Grid,
            #[cfg(feature = "gecko")]
            Display::WebkitInlineBox => Display::WebkitBox,

            // Special handling for contents and list-item on the root
            // element for Gecko.
            #[cfg(feature = "gecko")]
            Display::Contents | Display::ListItem if _is_root_element => Display::Block,

            // These are not changed by blockification.
            Display::None |
            Display::Block |
            Display::Flex |
            Display::ListItem |
            Display::Table => *self,

            #[cfg(feature = "gecko")]
            Display::Contents |
            Display::FlowRoot |
            Display::Grid |
            Display::WebkitBox => *self,

            // Everything else becomes block.
            _ => Display::Block,
        }

    }

    /// Convert this display into an inline-outside display.
    ///
    /// Ideally it should implement spec: https://drafts.csswg.org/css-display/#inlinify
    /// but the spec isn't stable enough, so we copy what Gecko does for now.
    #[cfg(feature = "gecko")]
    pub fn inlinify(&self) -> Self {
        match *self {
            Display::Block |
            Display::FlowRoot => Display::InlineBlock,
            Display::Table => Display::InlineTable,
            Display::Flex => Display::InlineFlex,
            Display::Grid => Display::InlineGrid,
            Display::MozBox => Display::MozInlineBox,
            Display::MozStack => Display::MozInlineStack,
            Display::WebkitBox => Display::WebkitInlineBox,
            other => other,
        }
    }

    #[cfg(feature = "servo")]
    #[inline]
    /// Custom cascade for the `display` property in servo
    pub fn cascade_property_custom(
        _declaration: &PropertyDeclaration,
        context: &mut Context
    ) {
        longhands::_servo_display_for_hypothetical_box::derive_from_display(context);
        longhands::_servo_text_decorations_in_effect::derive_from_display(context);
    }
}

/// A specified value for the `vertical-align` property.
pub type VerticalAlign = GenericVerticalAlign<LengthOrPercentage>;

impl Parse for VerticalAlign {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_quirky(context, i, AllowQuirks::Yes)) {
            return Ok(GenericVerticalAlign::Length(lop));
        }

        try_match_ident_ignore_ascii_case! { input,
            "baseline" => Ok(GenericVerticalAlign::Baseline),
            "sub" => Ok(GenericVerticalAlign::Sub),
            "super" => Ok(GenericVerticalAlign::Super),
            "top" => Ok(GenericVerticalAlign::Top),
            "text-top" => Ok(GenericVerticalAlign::TextTop),
            "middle" => Ok(GenericVerticalAlign::Middle),
            "bottom" => Ok(GenericVerticalAlign::Bottom),
            "text-bottom" => Ok(GenericVerticalAlign::TextBottom),
            #[cfg(feature = "gecko")]
            "-moz-middle-with-baseline" => {
                Ok(GenericVerticalAlign::MozMiddleWithBaseline)
            },
        }
    }
}

/// https://drafts.csswg.org/css-animations/#animation-iteration-count
pub type AnimationIterationCount = GenericAnimationIterationCount<Number>;

impl Parse for AnimationIterationCount {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut ::cssparser::Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("infinite")).is_ok() {
            return Ok(GenericAnimationIterationCount::Infinite)
        }

        let number = Number::parse_non_negative(context, input)?;
        Ok(GenericAnimationIterationCount::Number(number))
    }
}

impl AnimationIterationCount {
    /// Returns the value `1.0`.
    #[inline]
    pub fn one() -> Self {
        GenericAnimationIterationCount::Number(Number::new(1.0))
    }
}

/// A value for the `animation-name` property.
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct AnimationName(pub Option<KeyframesName>);

impl AnimationName {
    /// Get the name of the animation as an `Atom`.
    pub fn as_atom(&self) -> Option<&Atom> {
        self.0.as_ref().map(|n| n.as_atom())
    }

    /// Returns the `none` value.
    pub fn none() -> Self {
        AnimationName(None)
    }
}

impl ToCss for AnimationName {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self.0 {
            Some(ref name) => name.to_css(dest),
            None => dest.write_str("none"),
        }
    }
}

impl Parse for AnimationName {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(name) = input.try(|input| KeyframesName::parse(context, input)) {
            return Ok(AnimationName(Some(name)));
        }

        input.expect_ident_matching("none")?;
        Ok(AnimationName(None))
    }
}

define_css_keyword_enum! { ScrollSnapType:
    "none" => None,
    "mandatory" => Mandatory,
    "proximity" => Proximity,
}
add_impls_for_keyword_enum!(ScrollSnapType);

define_css_keyword_enum! { OverscrollBehavior:
    "auto" => Auto,
    "contain" => Contain,
    "none" => None,
}
add_impls_for_keyword_enum!(OverscrollBehavior);

define_css_keyword_enum! { OverflowClipBox:
    "padding-box" => PaddingBox,
    "content-box" => ContentBox,
}
add_impls_for_keyword_enum!(OverflowClipBox);

#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
/// Provides a rendering hint to the user agent,
/// stating what kinds of changes the author expects
/// to perform on the element
///
/// <https://drafts.csswg.org/css-will-change/#will-change>
pub enum WillChange {
    /// Expresses no particular intent
    Auto,
    #[css(comma, iterable)]
    /// <custom-ident>
    AnimateableFeatures(Box<[CustomIdent]>),
}

impl WillChange {
    #[inline]
    /// Get default value of `will-change` as `auto`
    pub fn auto() -> WillChange {
        WillChange::Auto
    }
}

impl Parse for WillChange {
    /// auto | <animateable-feature>#
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<WillChange, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
            return Ok(WillChange::Auto);
        }

        let custom_idents = input.parse_comma_separated(|i| {
            let location = i.current_source_location();
            CustomIdent::from_ident(location, i.expect_ident()?, &[
                "will-change",
                "none",
                "all",
                "auto",
            ])
        })?;

        Ok(WillChange::AnimateableFeatures(custom_idents.into_boxed_slice()))
    }
}

bitflags! {
    #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
    #[derive(ToComputedValue)]
    /// These constants match Gecko's `NS_STYLE_TOUCH_ACTION_*` constants.
    pub struct TouchAction: u8 {
        /// `none` variant
        const TOUCH_ACTION_NONE = 1 << 0;
        /// `auto` variant
        const TOUCH_ACTION_AUTO = 1 << 1;
        /// `pan-x` variant
        const TOUCH_ACTION_PAN_X = 1 << 2;
        /// `pan-y` variant
        const TOUCH_ACTION_PAN_Y = 1 << 3;
        /// `manipulation` variant
        const TOUCH_ACTION_MANIPULATION = 1 << 4;
    }
}

impl TouchAction {
    #[inline]
    /// Get default `touch-action` as `auto`
    pub fn auto() -> TouchAction {
        TouchAction::TOUCH_ACTION_AUTO
    }
}

impl ToCss for TouchAction {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            TouchAction::TOUCH_ACTION_NONE => dest.write_str("none"),
            TouchAction::TOUCH_ACTION_AUTO => dest.write_str("auto"),
            TouchAction::TOUCH_ACTION_MANIPULATION => dest.write_str("manipulation"),
            _ if self.contains(TouchAction::TOUCH_ACTION_PAN_X | TouchAction::TOUCH_ACTION_PAN_Y) => {
                dest.write_str("pan-x pan-y")
            },
            _ if self.contains(TouchAction::TOUCH_ACTION_PAN_X) => {
                dest.write_str("pan-x")
            },
            _ if self.contains(TouchAction::TOUCH_ACTION_PAN_Y) => {
                dest.write_str("pan-y")
            },
            _ => panic!("invalid touch-action value"),
        }
    }
}

impl Parse for TouchAction {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<TouchAction, ParseError<'i>> {
        try_match_ident_ignore_ascii_case! { input,
            "auto" => Ok(TouchAction::TOUCH_ACTION_AUTO),
            "none" => Ok(TouchAction::TOUCH_ACTION_NONE),
            "manipulation" => Ok(TouchAction::TOUCH_ACTION_MANIPULATION),
            "pan-x" => {
                if input.try(|i| i.expect_ident_matching("pan-y")).is_ok() {
                    Ok(TouchAction::TOUCH_ACTION_PAN_X | TouchAction::TOUCH_ACTION_PAN_Y)
                } else {
                    Ok(TouchAction::TOUCH_ACTION_PAN_X)
                }
            },
            "pan-y" => {
                if input.try(|i| i.expect_ident_matching("pan-x")).is_ok() {
                    Ok(TouchAction::TOUCH_ACTION_PAN_X | TouchAction::TOUCH_ACTION_PAN_Y)
                } else {
                    Ok(TouchAction::TOUCH_ACTION_PAN_Y)
                }
            },
        }
    }
}

#[cfg(feature = "gecko")]
impl_bitflags_conversions!(TouchAction);

/// Asserts that all touch-action matches its NS_STYLE_TOUCH_ACTION_* value.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_touch_action_matches() {
    use gecko_bindings::structs;

    macro_rules! check_touch_action {
        ( $( $a:ident => $b:path),*, ) => {
            $(
                debug_assert_eq!(structs::$a as u8, $b.bits());
            )*
        }
    }

    check_touch_action! {
        NS_STYLE_TOUCH_ACTION_NONE => TouchAction::TOUCH_ACTION_NONE,
        NS_STYLE_TOUCH_ACTION_AUTO => TouchAction::TOUCH_ACTION_AUTO,
        NS_STYLE_TOUCH_ACTION_PAN_X => TouchAction::TOUCH_ACTION_PAN_X,
        NS_STYLE_TOUCH_ACTION_PAN_Y => TouchAction::TOUCH_ACTION_PAN_Y,
        NS_STYLE_TOUCH_ACTION_MANIPULATION => TouchAction::TOUCH_ACTION_MANIPULATION,
    }
}
