//! https://developer.mozilla.org/en-US/docs/Web/HTML/Element.
//! https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes

use crate::HtmlElementKind;
use pinwheel_elements_macro::element;

// Main Root.

element!(
	/// The HTML `<html>` element represents the root (top-level element) of an HTML document, so it is also referred to as the root element. All other elements must be descendants of this element.
	namespace = "html",
	tag = "html",
);

// Document metadata.

element!(
	/// The HTML `<base>` element specifies the base URL to use for all relative URLs in a document.
	namespace = "html",
	tag = "base",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML `<head>` element contains machine-readable information (metadata) about the document, like its title, scripts, and style sheets.
	namespace = "html",
	tag = "head",
);

element!(
	/// The HTML External Resource Link element (`<link>`) specifies relationships between the current document and an external resource. This element is most commonly used to link to CSS, but is also used to establish site icons (both \"favicon\" style icons and icons for the home screen and apps on mobile devices) among other things.
	namespace = "html",
	tag = "link",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML `<meta>` element represents Metadata that cannot be represented by other HTML meta-related elements, like base, link, script, style or title.
	namespace = "html",
	tag = "meta",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML `<style>` element contains style information for a document, or part of a document.
	namespace = "html",
	tag = "style",
	kind = HtmlElementKind::RawText,
);

element!(
	/// The HTML Title element (`<title>`) defines the document's title that is shown in a Browser's title bar or a page's tab.
	namespace = "html",
	tag = "title",
	kind = HtmlElementKind::EscapableRawText,
);

// Sectioning root.

element!(
	/// The HTML `<body> Element represents the content of an HTML document. There can be only one <body>` element in a document.
	namespace = "html",
	tag = "body",
);

// Content sectioning

element!(
	/// The HTML `<address>` element indicates that the enclosed HTML provides contact information for a person or people, or for an organization.
	namespace = "html",
	tag = "address",
);

element!(
	/// The HTML `<article>` element represents a self-contained composition in a document, page, application, or site, which is intended to be independently distributable or reusable (e.g., in syndication).
	namespace = "html",
	tag = "article",
);

element!(
	/// The HTML `<aside>` element represents a portion of a document whose content is only indirectly related to the document's main content.
	namespace = "html",
	tag = "aside",
);

element!(
	/// The HTML `<footer> element represents a footer for its nearest sectioning content or sectioning root element. A <footer>` typically contains information about the author of the section, copyright data or links to related documents.
	namespace = "html",
	tag = "footer",
);

element!(
	/// The HTML `<header>` element represents introductory content, typically a group of introductory or navigational aids. It may contain some heading elements but also a logo, a search form, an author name, and other elements.
	namespace = "html",
	tag = "header",
);

element!(
	/// The HTML `<h1>-<h6> elements represent six levels of section headings. <h1> is the highest section level and <h6>` is the lowest.
	namespace = "html",
	tag = "h1",
);

element!(
	/// The HTML `<h1>-<h6> elements represent six levels of section headings. <h1> is the highest section level and <h6>` is the lowest.
	namespace = "html",
	tag = "h2",
);

element!(
	/// The HTML `<h1>-<h6> elements represent six levels of section headings. <h1> is the highest section level and <h6>` is the lowest.
	namespace = "html",
	tag = "h3",
);

element!(
	/// The HTML `<h1>-<h6> elements represent six levels of section headings. <h1> is the highest section level and <h6>` is the lowest.
	namespace = "html",
	tag = "h4",
);

element!(
	/// The HTML `<h1>-<h6> elements represent six levels of section headings. <h1> is the highest section level and <h6>` is the lowest.
	namespace = "html",
	tag = "h5",
);

element!(
	/// The HTML `<h1>-<h6> elements represent six levels of section headings. <h1> is the highest section level and <h6>` is the lowest.
	namespace = "html",
	tag = "h6",
);

element!(
	/// The HTML `<main>` element represents the dominant content of the body of a document. The main content area consists of content that is directly related to or expands upon the central topic of a document, or the central functionality of an application.
	namespace = "html",
	tag = "main",
);

element!(
	/// The HTML `<nav>` element represents a section of a page whose purpose is to provide navigation links, either within the current document or to other documents. Common examples of navigation sections are menus, tables of contents, and indexes.
	namespace = "html",
	tag = "nav",
);

element!(
	/// The HTML `<section>` element represents a generic standalone section of a document, which doesn't have a more specific semantic element to represent it.
	namespace = "html",
	tag = "section",
);

// Text content.

element!(
	/// The HTML `<blockquote>` Element (or HTML Block Quotation Element) indicates that the enclosed text is an extended quotation. Usually, this is rendered visually by indentation (see Notes for how to change it). A URL for the source of the quotation may be given using the cite attribute, while a text representation of the source can be given using the cite element.
	namespace = "html",
	tag = "blockquote",
);

element!(
	/// The HTML `<dd>` element provides the description, definition, or value for the preceding term (dt) in a description list (dl).
	namespace = "html",
	tag = "dd",
);

element!(
	/// The HTML Content Division element (`<div>`) is the generic container for flow content. It has no effect on the content or layout until styled in some way using CSS (e.g. styling is directly applied to it, or some kind of layout model like Flexbox is applied to its parent element).
	namespace = "html",
	tag = "div",
	element = web_sys::HtmlDivElement,
);

element!(
	/// The HTML `<dl>` element represents a description list. The element encloses a list of groups of terms (specified using the dt element) and descriptions (provided by dd elements). Common uses for this element are to implement a glossary or to display metadata (a list of key-value pairs).
	namespace = "html",
	tag = "dl",
);

element!(
	/// The HTML `<dt>` element specifies a term in a description or definition list, and as such must be used inside a dl element.
	namespace = "html",
	tag = "dt",
);

element!(
	/// The HTML `<figcaption>` or Figure Caption element represents a caption or legend describing the rest of the contents of its parent figure element.
	namespace = "html",
	tag = "figcaption",
);

element!(
	/// The HTML `<figure>` (Figure With Optional Caption) element represents self-contained content, potentially with an optional caption, which is specified using the figcaption element.
	namespace = "html",
	tag = "figure",
);

element!(
	/// The HTML `<hr>` element represents a thematic break between paragraph-level elements: for example, a change of scene in a story, or a shift of topic within a section.
	namespace = "html",
	tag = "hr",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML `<li>` element is used to represent an item in a list.
	namespace = "html",
	tag = "li",
);

element!(
	/// The HTML `<ol>` element represents an ordered list of items - typically rendered as a numbered list.
	namespace = "html",
	tag = "ol",
);

element!(
	/// The HTML `<p>` element represents a paragraph.
	namespace = "html",
	tag = "p",
);

element!(
	/// The HTML `<pre>` element represents preformatted text which is to be presented exactly as written in the HTML file.
	namespace = "html",
	tag = "pre",
);

element!(
	/// The HTML `<ul>` element represents an unordered list of items, typically rendered as a bulleted list.
	namespace = "html",
	tag = "ul",
);

// Inline text semantics.

element!(
	/// The HTML `<a>` element (or anchor element), with its href attribute, creates a hyperlink to web pages, files, email addresses, locations in the same page, or anything else a URL can address.
	namespace = "html",
	tag = "a",
	attributes = {
		download,
		href,
		hreflang,
		media,
		ping,
		referrerpolicy,
		rel,
		shape,
		target,
	},
);

element!(
	/// The HTML Abbreviation element (`<abbr>`) represents an abbreviation or acronym; the optional title attribute can provide an expansion or description for the abbreviation.
	namespace = "html",
	tag = "abbr",
);

element!(
	/// The HTML Bring Attention To element (`<b>`) is used to draw the reader's attention to the element's contents, which are not otherwise granted special importance.
	namespace = "html",
	tag = "b",
);

element!(
	/// The HTML Bidirectional Isolate element (`<bdi>`)  tells the browser's bidirectional algorithm to treat the text it contains in isolation from its surrounding text.
	namespace = "html",
	tag = "bdi",
);

element!(
	/// The HTML Bidirectional Text Override element (`<bdo>`) overrides the current directionality of text, so that the text within is rendered in a different direction.
	namespace = "html",
	tag = "bdo",
);

element!(
	/// The HTML `<br>` element produces a line break in text (carriage-return). It is useful for writing a poem or an address, where the division of lines is significant.
	namespace = "html",
	tag = "br",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML Citation element (`<cite>`) is used to describe a reference to a cited creative work, and must include the title of that work.
	namespace = "html",
	tag = "cite",
);

element!(
	/// The HTML `<code>` element displays its contents styled in a fashion intended to indicate that the text is a short fragment of computer code.
	namespace = "html",
	tag = "code",
);

element!(
	/// The HTML `<data>` element links a given piece of content with a machine-readable translation. If the content is time- or date-related, the time element must be used.
	namespace = "html",
	tag = "data",
);

element!(
	/// The HTML Definition element (`<dfn>`) is used to indicate the term being defined within the context of a definition phrase or sentence.
	namespace = "html",
	tag = "fdn",
);

element!(
	/// The HTML `<em> element marks text that has stress emphasis. The <em>` element can be nested, with each level of nesting indicating a greater degree of emphasis.
	namespace = "html",
	tag = "em",
);

element!(
	/// The HTML Idiomatic Text element (`<i>`) represents a range of text that is set off from the normal text for some reason, such as idiomatic text, technical terms, taxonomical designations, among others.
	namespace = "html",
	tag = "i",
);

element!(
	/// The HTML Keyboard Input element (`<kbd>`) represents a span of inline text denoting textual user input from a keyboard, voice input, or any other text entry device.
	namespace = "html",
	tag = "kbd",
);

element!(
	/// The HTML Mark Text element (`<mark>`) represents text which is marked or highlighted for reference or notation purposes, due to the marked passage's relevance or importance in the enclosing context.
	namespace = "html",
	tag = "mark",
);

element!(
	/// The HTML `<q>` element indicates that the enclosed text is a short inline quotation. Most modern browsers implement this by surrounding the text in quotation marks.
	namespace = "html",
	tag = "q",
);

element!(
	/// The HTML Ruby Base (`<rb>`) element is used to delimit the base text component of a  ruby annotation, i.e. the text that is being annotated.
	namespace = "html",
	tag = "rb",
);

element!(
	/// The HTML Ruby Fallback Parenthesis (`<rp>`) element is used to provide fall-back parentheses for browsers that do not support display of ruby annotations using the ruby element.
	namespace = "html",
	tag = "rp",
);

element!(
	/// The HTML Ruby Text (`<rt>) element specifies the ruby text component of a ruby annotation, which is used to provide pronunciation, translation, or transliteration information for East Asian typography. The <rt>` element must always be contained within a ruby element.
	namespace = "html",
	tag = "rt",
);

element!(
	/// The HTML Ruby Text Container (`<rtc>`) element embraces semantic annotations of characters presented in a ruby of rb elements used inside of ruby element. rb elements can have both pronunciation (rt) and semantic (rtc) annotations.
	namespace = "html",
	tag = "rtc",
);

element!(
	/// The HTML `<ruby>` element represents small annotations that are rendered above, below, or next to base text, usually used for showing the pronunciation of East Asian characters. It can also be used for annotating other kinds of text, but this usage is less common.
	namespace = "html",
	tag = "ruby",
);

element!(
	/// The HTML `<s> element renders text with a strikethrough, or a line through it. Use the <s> element to represent things that are no longer relevant or no longer accurate. However, <s>` is not appropriate when indicating document edits; for that, use the del and ins elements, as appropriate.
	namespace = "html",
	tag = "s",
);

element!(
	/// The HTML Sample Element (`<samp>`) is used to enclose inline text which represents sample (or quoted) output from a computer program.
	namespace = "html",
	tag = "samp",
);

element!(
	/// The HTML `<small>` element represents side-comments and small print, like copyright and legal text, independent of its styled presentation. By default, it renders text within it one font-size smaller, such as from small to x-small.
	namespace = "html",
	tag = "small",
);

element!(
	/// The HTML `<span>` element is a generic inline container for phrasing content, which does not inherently represent anything. It can be used to group elements for styling purposes (using the class or id attributes), or because they share attribute values, such as lang.
	namespace = "html",
	tag = "span",
);

element!(
	/// The HTML Strong Importance Element (`<strong>`) indicates that its contents have strong importance, seriousness, or urgency. Browsers typically render the contents in bold type.
	namespace = "html",
	tag = "strong",
);

element!(
	/// The HTML Subscript element (`<sub>`) specifies inline text which should be displayed as subscript for solely typographical reasons.
	namespace = "html",
	tag = "sub",
);

element!(
	/// The HTML Superscript element (`<sup>`) specifies inline text which is to be displayed as superscript for solely typographical reasons.
	namespace = "html",
	tag = "sup",
);

element!(
	/// The HTML `<time>` element represents a specific period in time.
	namespace = "html",
	tag = "time",
);

element!(
	/// The HTML Unarticulated Annotation element (`<u>`) represents a span of inline text which should be rendered in a way that indicates that it has a non-textual annotation.
	namespace = "html",
	tag = "u",
);

element!(
	/// The HTML Variable element (`<var>`) represents the name of a variable in a mathematical expression or a programming context.
	namespace = "html",
	tag = "var",
);

element!(
	/// The HTML `<wbr>` element represents a word break opportunity - a position within text where the browser may optionally break a line, though its line-breaking rules would not otherwise create a break at that location.
	namespace = "html",
	tag = "wbr",
	kind = HtmlElementKind::Void,
);

// Image and multimedia.

element!(
	/// The HTML `<area>` element defines an area inside an image map that has predefined clickable areas. An image map allows geometric areas on an image to be associated with Hyperlink.
	namespace = "html",
	tag = "area",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML `<audio>` element is used to embed sound content in documents. It may contain one or more audio sources, represented using the src attribute or the source element: the browser will choose the most suitable one. It can also be the destination for streamed media, using a MediaStream.
	namespace = "html",
	tag = "audio",
);

element!(
	/// The HTML `<img>` element embeds an image into the document.
	namespace = "html",
	tag = "img",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML `<map>` element is used with area elements to define an image map (a clickable link area).
	namespace = "html",
	tag = "map",
);

element!(
	/// The HTML `<track>` element is used as a child of the media elements, audio and video. It lets you specify timed text tracks (or time-based data), for example to automatically handle subtitles.
	namespace = "html",
	tag = "track",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML Video element (`<video>) embeds a media player which supports video playback into the document. You can use <video>` for audio content as well, but the audio element may provide a more appropriate user experience.
	namespace = "html",
	tag = "video",
);

// Embedded content.

element!(
	/// The HTML `<embed>` element embeds external content at the specified point in the document. This content is provided by an external application or other source of interactive content such as a browser plug-in.
	namespace = "html",
	tag = "embed",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML Inline Frame element (`<iframe>`) represents a nested browsing context, embedding another HTML page into the current one.
	namespace = "html",
	tag = "iframe",
);

element!(
	/// The HTML `<object>` element represents an external resource, which can be treated as an image, a nested browsing context, or a resource to be handled by a plugin.
	namespace = "html",
	tag = "object",
);

element!(
	/// The HTML `<param>` element defines parameters for an object element.
	namespace = "html",
	tag = "param",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML `<picture>` element contains zero or more source elements and one img element to offer alternative versions of an image for different display/device scenarios.
	namespace = "html",
	tag = "picture",
);

element!(
	/// The HTML Portal element (`<portal>`) enables the embedding of another HTML page into the current one for the purposes of allowing smoother navigation into new pages.
	namespace = "html",
	tag = "portal",
);

element!(
	/// The HTML `<source>` element specifies multiple media resources for the picture, the audio element, or the video element.
	namespace = "html",
	tag = "source",
	kind = HtmlElementKind::Void,
);

// SVG and MathML.

element!(
	/// The svg element is a container that defines a new coordinate system and viewport. It is used as the outermost element of SVG documents, but it can also be used to embed an SVG fragment inside an SVG or HTML document.
	namespace = "html",
	tag = "svg",
);

element!(
	/// The top-level element in MathML is `<math>. Every valid MathML instance must be wrapped in <math> tags. In addition you must not nest a second <math>` element in another, but you can have an arbitrary number of other child elements in it.
	namespace = "html",
	tag = "math",
);

// Scripting

element!(
	/// Use the HTML `<canvas>` element with either the canvas scripting API or the WebGL API to draw graphics and animations.
	namespace = "html",
	tag = "canvas",
);

element!(
	/// The HTML `<noscript>` element defines a section of HTML to be inserted if a script type on the page is unsupported or if scripting is currently turned off in the browser.
	namespace = "html",
	tag = "noscript",
);

element!(
	/// The HTML `<script>` element is used to embed executable code or data; this is typically used to embed or refer to JavaScript code.
	namespace = "html",
	tag = "script",
	kind = HtmlElementKind::RawText,
);

// Demarcating edits.

element!(
	/// The HTML `<del>` element represents a range of text that has been deleted from a document.
	namespace = "html",
	tag = "del",
);

element!(
	/// The HTML `<ins>` element represents a range of text that has been added to a document.
	namespace = "html",
	tag = "ins",
);

// Table content.

element!(
	/// The HTML `<caption>` element specifies the caption (or title) of a table.
	namespace = "html",
	tag = "caption",
);

element!(
	/// The HTML `<col>` element defines a column within a table and is used for defining common semantics on all common cells. It is generally found within a colgroup element.
	namespace = "html",
	tag = "col",
	kind = HtmlElementKind::Void,
);

element!(
	/// The HTML `<colgroup>` element defines a group of columns within a table.
	namespace = "html",
	tag = "colgroup",
);

element!(
	/// The HTML `<table>` element represents tabular data - that is, information presented in a two-dimensional table comprised of rows and columns of cells containing data.
	namespace = "html",
	tag = "table",
);

element!(
	/// The HTML Table Body element (`<tbody>`) encapsulates a set of table rows (tr elements), indicating that they comprise the body of the table (table).
	namespace = "html",
	tag = "tbody",
);

element!(
	/// The HTML `<td>` element defines a cell of a table that contains data. It participates in the table model.
	namespace = "html",
	tag = "td",
);

element!(
	/// The HTML `<tfoot>` element defines a set of rows summarizing the columns of the table.
	namespace = "html",
	tag = "tfoot",
);

element!(
	/// The HTML `<th>` element defines a cell as header of a group of table cells. The exact nature of this group is defined by the scope and headers attributes.
	namespace = "html",
	tag = "th",
);

element!(
	/// The HTML `<thead>` element defines a set of rows defining the head of the columns of the table.
	namespace = "html",
	tag = "thead",
);

element!(
	/// The HTML `<tr>` element defines a row of cells in a table. The row's cells can then be established using a mix of td (data cell) and th (header cell) elements.
	namespace = "html",
	tag = "tr",
);

// Forms.

element!(
	/// The HTML `<button>` element represents a clickable button, used to submit forms or anywhere in a document for accessible, standard button functionality.
	namespace = "html",
	tag = "button",
	attributes = {
		autofocus,
		disabled?,
		form,
		formaction,
		formenctype,
		formmethod,
		formnovalidate,
		formtarget,
		name,
		// type,
		value,
	},
);

element!(
	/// The HTML `<datalist>` element contains a set of option elements that represent the permissible or recommended options available to choose from within other controls.
	namespace = "html",
	tag = "datalist",
);

element!(
	/// The HTML `<fieldset>` element is used to group several controls as well as labels (label) within a web form.
	namespace = "html",
	tag = "fieldset",
);

element!(
	/// The HTML `<form>` element represents a document section containing interactive controls for submitting information.
	namespace = "html",
	tag = "form",
	attributes = {
		accept,
		accept_charset,
		action,
		autocomplete,
		enctype,
		method,
		name,
		novalidate?,
		target,
	},
);

element!(
	/// The HTML `<input>` element is used to create interactive controls for web-based forms in order to accept data from the user; a wide variety of types of input data and control widgets are available, depending on the device and user agent.
	namespace = "html",
	tag = "input",
	kind = HtmlElementKind::Void,
	attributes = {
		accept,
		alt,
		autocomplete,
		autofocus,
		capture,
		checked,
		dirname,
		disabled?,
		form,
		formaction,
		formenctype,
		formmethod,
		formnovalidate,
		formtarget,
		height,
		list,
		max,
		maxlength,
		minlength,
		min,
		multiple,
		name,
		pattern,
		placeholder,
		readonly?,
		required?,
		size,
		src,
		step,
		// type,
		usemap,
		value,
		width,
	},
	events = {
		input: InputEvent,
	},
);

element!(
	/// The HTML `<label>` element represents a caption for an item in a user interface.
	namespace = "html",
	tag = "label",
);

element!(
	/// The HTML `<legend>` element represents a caption for the content of its parent fieldset.
	namespace = "html",
	tag = "legend",
);

element!(
	/// The HTML `<meter>` element represents either a scalar value within a known range or a fractional value.
	namespace = "html",
	tag = "meter",
);

element!(
	/// The HTML `<optgroup>` element creates a grouping of options within a select element.
	namespace = "html",
	tag = "optgroup",
);

element!(
	/// The HTML `<option> element is used to define an item contained in a select, an optgroup, or a datalist element. As such, <option>` can represent menu items in popups and other lists of items in an HTML document.
	namespace = "html",
	tag = "option",
);

element!(
	/// The HTML Output element (`<output>`) is a container element into which a site or app can inject the results of a calculation or the outcome of a user action.
	namespace = "html",
	tag = "output",
);

element!(
	/// The HTML `<progress>` element displays an indicator showing the completion progress of a task, typically displayed as a progress bar.
	namespace = "html",
	tag = "progress",
);

element!(
	/// The HTML `<select>` element represents a control that provides a menu of options
	namespace = "html",
	tag = "select",
	events = {
		change: InputEvent,
		input: InputEvent,
	},
);

element!(
	/// The HTML `<textarea>` element represents a multi-line plain-text editing control, useful when you want to allow users to enter a sizeable amount of free-form text, for example a comment on a review or feedback form.
	namespace = "html",
	tag = "textarea",
	kind = HtmlElementKind::EscapableRawText,
);

// Interactive elements.

element!(
	/// The HTML Details Element (`<details>`) creates a disclosure widget in which information is visible only when the widget is toggled into an \"open\" state.
	namespace = "html",
	tag = "details",
);

element!(
	/// The HTML `<dialog>` element represents a dialog box or other interactive component, such as a dismissible alert, inspector, or subwindow.
	namespace = "html",
	tag = "dialog",
);

element!(
	/// The HTML `<menu>` element represents a group of commands that a user can perform or activate. This includes both list menus, which might appear across the top of a screen, as well as context menus, such as those that might appear underneath a button after it has been clicked.
	namespace = "html",
	tag = "menu",
);

element!(
	/// The HTML Disclosure Summary element (`<summary>`) element specifies a summary, caption, or legend for a details element's disclosure box.
	namespace = "html",
	tag = "summary",
);

// Web components.

element!(
	/// The HTML `<slot>` element - part of the Web Components technology suite - is a placeholder inside a web component that you can fill with your own markup, which lets you create separate DOM trees and present them together.
	namespace = "html",
	tag = "slot",
);

element!(
	/// The HTML Content Template (`<template>`) element is a mechanism for holding HTML that is not to be rendered immediately when a page is loaded but may be instantiated subsequently during runtime using JavaScript.
	namespace = "html",
	tag = "template",
	kind = HtmlElementKind::Template,
);

// Obsoltete and deprecated elements.

// These items are unimplemented.

pub mod style {
	pub const ALIGN_CONTENT: &str = "align-content";
	pub const ALIGN_ITEMS: &str = "align-items";
	pub const ALIGN_SELF: &str = "align-self";
	pub const ALIGNMENT_BASELINE: &str = "alignment-baseline";
	pub const ANIMATION: &str = "animation";
	pub const ANIMATION_DELAY: &str = "animation-delay";
	pub const ANIMATION_DIRECTION: &str = "animation-direction";
	pub const ANIMATION_DURATION: &str = "animation-duration";
	pub const ANIMATION_FILL_MODE: &str = "animation-fill-mode";
	pub const ANIMATION_ITERATION_COUNT: &str = "animation-iteration-count";
	pub const ANIMATION_NAME: &str = "animation-name";
	pub const ANIMATION_PLAY_STATE: &str = "animation-play-state";
	pub const ANIMATION_TIMING_FUNCTION: &str = "animation-timing-function";
	pub const BACKFACE_VISIBILITY: &str = "backface-visibility";
	pub const BACKGROUND: &str = "background";
	pub const BACKGROUND_ATTACHMENT: &str = "background-attachment";
	pub const BACKGROUND_CLIP: &str = "background-clip";
	pub const BACKGROUND_COLOR: &str = "background-color";
	pub const BACKGROUND_IMAGE: &str = "background-image";
	pub const BACKGROUND_ORIGIN: &str = "background-origin";
	pub const BACKGROUND_POSITION: &str = "background-position";
	pub const BACKGROUND_POSITION_X: &str = "background-position-x";
	pub const BACKGROUND_POSITION_Y: &str = "background-position-y";
	pub const BACKGROUND_REPEAT: &str = "background-repeat";
	pub const BACKGROUND_SIZE: &str = "background-size";
	pub const BASELINE_SHIFT: &str = "baseline-shift";
	pub const BLOCK_SIZE: &str = "block-size";
	pub const BORDER: &str = "border";
	pub const BORDER_BLOCK_END: &str = "border-block-end";
	pub const BORDER_BLOCK_END_COLOR: &str = "border-block-end-color";
	pub const BORDER_BLOCK_END_STYLE: &str = "border-block-end-style";
	pub const BORDER_BLOCK_END_WIDTH: &str = "border-block-end-width";
	pub const BORDER_BLOCK_START: &str = "border-block-start";
	pub const BORDER_BLOCK_START_COLOR: &str = "border-block-start-color";
	pub const BORDER_BLOCK_START_STYLE: &str = "border-block-start-style";
	pub const BORDER_BLOCK_START_WIDTH: &str = "border-block-start-width";
	pub const BORDER_BOTTOM: &str = "border-bottom";
	pub const BORDER_BOTTOM_COLOR: &str = "border-bottom-color";
	pub const BORDER_BOTTOM_LEFT_RADIUS: &str = "border-bottom-left-radius";
	pub const BORDER_BOTTOM_RIGHT_RADIUS: &str = "border-bottom-right-radius";
	pub const BORDER_BOTTOM_STYLE: &str = "border-bottom-style";
	pub const BORDER_BOTTOM_WIDTH: &str = "border-bottom-width";
	pub const BORDER_COLLAPSE: &str = "border-collapse";
	pub const BORDER_COLOR: &str = "border-color";
	pub const BORDER_IMAGE: &str = "border-image";
	pub const BORDER_IMAGE_OUTSET: &str = "border-image-outset";
	pub const BORDER_IMAGE_REPEAT: &str = "border-image-repeat";
	pub const BORDER_IMAGE_SLICE: &str = "border-image-slice";
	pub const BORDER_IMAGE_SOURCE: &str = "border-image-source";
	pub const BORDER_IMAGE_WIDTH: &str = "border-image-width";
	pub const BORDER_INLINE_END: &str = "border-inline-end";
	pub const BORDER_INLINE_END_COLOR: &str = "border-inline-end-color";
	pub const BORDER_INLINE_END_STYLE: &str = "border-inline-end-style";
	pub const BORDER_INLINE_END_WIDTH: &str = "border-inline-end-width";
	pub const BORDER_INLINE_START: &str = "border-inline-start";
	pub const BORDER_INLINE_START_COLOR: &str = "border-inline-start-color";
	pub const BORDER_INLINE_START_STYLE: &str = "border-inline-start-style";
	pub const BORDER_INLINE_START_WIDTH: &str = "border-inline-start-width";
	pub const BORDER_LEFT: &str = "border-left";
	pub const BORDER_LEFT_COLOR: &str = "border-left-color";
	pub const BORDER_LEFT_STYLE: &str = "border-left-style";
	pub const BORDER_LEFT_WIDTH: &str = "border-left-width";
	pub const BORDER_RADIUS: &str = "border-radius";
	pub const BORDER_RIGHT: &str = "border-right";
	pub const BORDER_RIGHT_COLOR: &str = "border-right-color";
	pub const BORDER_RIGHT_STYLE: &str = "border-right-style";
	pub const BORDER_RIGHT_WIDTH: &str = "border-right-width";
	pub const BORDER_SPACING: &str = "border-spacing";
	pub const BORDER_STYLE: &str = "border-style";
	pub const BORDER_TOP: &str = "border-top";
	pub const BORDER_TOP_COLOR: &str = "border-top-color";
	pub const BORDER_TOP_LEFT_RADIUS: &str = "border-top-left-radius";
	pub const BORDER_TOP_RIGHT_RADIUS: &str = "border-top-right-radius";
	pub const BORDER_TOP_STYLE: &str = "border-top-style";
	pub const BORDER_TOP_WIDTH: &str = "border-top-width";
	pub const BORDER_WIDTH: &str = "border-width";
	pub const BOTTOM: &str = "bottom";
	pub const BOX_SHADOW: &str = "box-shadow";
	pub const BOX_SIZING: &str = "box-sizing";
	pub const BREAK_AFTER: &str = "break-after";
	pub const BREAK_BEFORE: &str = "break-before";
	pub const BREAK_INSIDE: &str = "break-inside";
	pub const CAPTION_SIDE: &str = "caption-side";
	pub const CARET_COLOR: &str = "caret-color";
	pub const CLEAR: &str = "clear";
	pub const CLIP: &str = "clip";
	pub const CLIP_PATH: &str = "clip-path";
	pub const CLIP_RULE: &str = "clip-rule";
	pub const COLOR: &str = "color";
	pub const COLOR_INTERPOLATION: &str = "color-interpolation";
	pub const COLOR_INTERPOLATION_FILTERS: &str = "color-interpolation-filters";
	pub const COLUMN_COUNT: &str = "column-count";
	pub const COLUMN_FILL: &str = "column-fill";
	pub const COLUMN_GAP: &str = "column-gap";
	pub const COLUMN_RULE: &str = "column-rule";
	pub const COLUMN_RULE_COLOR: &str = "column-rule-color";
	pub const COLUMN_RULE_STYLE: &str = "column-rule-style";
	pub const COLUMN_RULE_WIDTH: &str = "column-rule-width";
	pub const COLUMN_SPAN: &str = "column-span";
	pub const COLUMN_WIDTH: &str = "column-width";
	pub const COLUMNS: &str = "columns";
	pub const CONTENT: &str = "content";
	pub const COUNTER_INCREMENT: &str = "counter-increment";
	pub const COUNTER_RESET: &str = "counter-reset";
	pub const CSS_FLOAT: &str = "css-float";
	pub const CSS_TEXT: &str = "css-text";
	pub const CURSOR: &str = "cursor";
	pub const DIRECTION: &str = "direction";
	pub const DISPLAY: &str = "display";
	pub const DOMINANT_BASELINE: &str = "dominant-baseline";
	pub const EMPTY_CELLS: &str = "empty-cells";
	pub const ENABLE_BACKGROUND: &str = "enable-background";
	pub const FILL: &str = "fill";
	pub const FILL_OPACITY: &str = "fill-opacity";
	pub const FILL_RULE: &str = "fill-rule";
	pub const FILTER: &str = "filter";
	pub const FLEX: &str = "flex";
	pub const FLEX_BASIS: &str = "flex-basis";
	pub const FLEX_DIRECTION: &str = "flex-direction";
	pub const FLEX_FLOW: &str = "flex-flow";
	pub const FLEX_GROW: &str = "flex-grow";
	pub const FLEX_SHRINK: &str = "flex-shrink";
	pub const FLEX_WRAP: &str = "flex-wrap";
	pub const FLOAT: &str = "float";
	pub const FLOOD_COLOR: &str = "flood-color";
	pub const FLOOD_OPACITY: &str = "flood-opacity";
	pub const FONT: &str = "font";
	pub const FONT_FAMILY: &str = "font-family";
	pub const FONT_FEATURE_SETTINGS: &str = "font-feature-settings";
	pub const FONT_KERNING: &str = "font-kerning";
	pub const FONT_SIZE: &str = "font-size";
	pub const FONT_SIZE_ADJUST: &str = "font-size-adjust";
	pub const FONT_STRETCH: &str = "font-stretch";
	pub const FONT_STYLE: &str = "font-style";
	pub const FONT_SYNTHESIS: &str = "font-synthesis";
	pub const FONT_VARIANT: &str = "font-variant";
	pub const FONT_VARIANT_CAPS: &str = "font-variant-caps";
	pub const FONT_VARIANT_EAST_ASIAN: &str = "font-variant-east-asian";
	pub const FONT_VARIANT_LIGATURES: &str = "font-variant-ligatures";
	pub const FONT_VARIANT_NUMERIC: &str = "font-variant-numeric";
	pub const FONT_VARIANT_POSITION: &str = "font-variant-position";
	pub const FONT_WEIGHT: &str = "font-weight";
	pub const GAP: &str = "gap";
	pub const GLYPH_ORIENTATION_HORIZONTAL: &str = "glyph-orientation-horizontal";
	pub const GLYPH_ORIENTATION_VERTICAL: &str = "glyph-orientation-vertical";
	pub const GRID: &str = "grid";
	pub const GRID_AREA: &str = "grid-area";
	pub const GRID_AUTO_COLUMNS: &str = "grid-auto-columns";
	pub const GRID_AUTO_FLOW: &str = "grid-auto-flow";
	pub const GRID_AUTO_ROWS: &str = "grid-auto-rows";
	pub const GRID_COLUMN: &str = "grid-column";
	pub const GRID_COLUMN_END: &str = "grid-column-end";
	pub const GRID_COLUMN_GAP: &str = "grid-column-gap";
	pub const GRID_COLUMN_START: &str = "grid-column-start";
	pub const GRID_GAP: &str = "grid-gap";
	pub const GRID_ROW: &str = "grid-row";
	pub const GRID_ROW_END: &str = "grid-row-end";
	pub const GRID_ROW_GAP: &str = "grid-row-gap";
	pub const GRID_ROW_START: &str = "grid-row-start";
	pub const GRID_TEMPLATE: &str = "grid-template";
	pub const GRID_TEMPLATE_AREAS: &str = "grid-template-areas";
	pub const GRID_TEMPLATE_COLUMNS: &str = "grid-template-columns";
	pub const GRID_TEMPLATE_ROWS: &str = "grid-template-rows";
	pub const HEIGHT: &str = "height";
	pub const HYPHENS: &str = "hyphens";
	pub const IMAGE_ORIENTATION: &str = "image-orientation";
	pub const IMAGE_RENDERING: &str = "image-rendering";
	pub const IME_MODE: &str = "ime-mode";
	pub const INLINE_SIZE: &str = "inline-size";
	pub const JUSTIFY_CONTENT: &str = "justify-content";
	pub const JUSTIFY_ITEMS: &str = "justify-items";
	pub const JUSTIFY_SELF: &str = "justify-self";
	pub const KERNING: &str = "kerning";
	pub const LAYOUT_GRID: &str = "layout-grid";
	pub const LAYOUT_GRID_CHAR: &str = "layout-grid-char";
	pub const LAYOUT_GRID_LINE: &str = "layout-grid-line";
	pub const LAYOUT_GRID_MODE: &str = "layout-grid-mode";
	pub const LAYOUT_GRID_TYPE: &str = "layout-grid-type";
	pub const LEFT: &str = "left";
	pub const LENGTH: &str = "length";
	pub const LETTER_SPACING: &str = "letter-spacing";
	pub const LIGHTING_COLOR: &str = "lighting-color";
	pub const LINE_BREAK: &str = "line-break";
	pub const LINE_HEIGHT: &str = "line-height";
	pub const LIST_STYLE: &str = "list-style";
	pub const LIST_STYLE_IMAGE: &str = "list-style-image";
	pub const LIST_STYLE_POSITION: &str = "list-style-position";
	pub const LIST_STYLE_TYPE: &str = "list-style-type";
	pub const MARGIN: &str = "margin";
	pub const MARGIN_BLOCK_END: &str = "margin-block-end";
	pub const MARGIN_BLOCK_START: &str = "margin-block-start";
	pub const MARGIN_BOTTOM: &str = "margin-bottom";
	pub const MARGIN_INLINE_END: &str = "margin-inline-end";
	pub const MARGIN_INLINE_START: &str = "margin-inline-start";
	pub const MARGIN_LEFT: &str = "margin-left";
	pub const MARGIN_RIGHT: &str = "margin-right";
	pub const MARGIN_TOP: &str = "margin-top";
	pub const MARKER: &str = "marker";
	pub const MARKER_END: &str = "marker-end";
	pub const MARKER_MID: &str = "marker-mid";
	pub const MARKER_START: &str = "marker-start";
	pub const MASK: &str = "mask";
	pub const MASK_COMPOSITE: &str = "mask-composite";
	pub const MASK_IMAGE: &str = "mask-image";
	pub const MASK_POSITION: &str = "mask-position";
	pub const MASK_REPEAT: &str = "mask-repeat";
	pub const MASK_SIZE: &str = "mask-size";
	pub const MASK_TYPE: &str = "mask-type";
	pub const MAX_BLOCK_SIZE: &str = "max-block-size";
	pub const MAX_HEIGHT: &str = "max-height";
	pub const MAX_INLINE_SIZE: &str = "max-inline-size";
	pub const MAX_WIDTH: &str = "max-width";
	pub const MIN_BLOCK_SIZE: &str = "min-block-size";
	pub const MIN_HEIGHT: &str = "min-height";
	pub const MIN_INLINE_SIZE: &str = "min-inline-size";
	pub const MIN_WIDTH: &str = "min-width";
	pub const MS_CONTENT_ZOOM_CHAINING: &str = "ms-content-zoom-chaining";
	pub const MS_CONTENT_ZOOM_LIMIT: &str = "ms-content-zoom-limit";
	pub const MS_CONTENT_ZOOM_LIMIT_MAX: &str = "ms-content-zoom-limit-max";
	pub const MS_CONTENT_ZOOM_LIMIT_MIN: &str = "ms-content-zoom-limit-min";
	pub const MS_CONTENT_ZOOM_SNAP: &str = "ms-content-zoom-snap";
	pub const MS_CONTENT_ZOOM_SNAP_POINTS: &str = "ms-content-zoom-snap-points";
	pub const MS_CONTENT_ZOOM_SNAP_TYPE: &str = "ms-content-zoom-snap-type";
	pub const MS_CONTENT_ZOOMING: &str = "ms-content-zooming";
	pub const MS_FLOW_FROM: &str = "ms-flow-from";
	pub const MS_FLOW_INTO: &str = "ms-flow-into";
	pub const MS_FONT_FEATURE_SETTINGS: &str = "ms-font-feature-settings";
	pub const MS_GRID_COLUMN: &str = "ms-grid-column";
	pub const MS_GRID_COLUMN_ALIGN: &str = "ms-grid-column-align";
	pub const MS_GRID_COLUMN_SPAN: &str = "ms-grid-column-span";
	pub const MS_GRID_COLUMNS: &str = "ms-grid-columns";
	pub const MS_GRID_ROW: &str = "ms-grid-row";
	pub const MS_GRID_ROW_ALIGN: &str = "ms-grid-row-align";
	pub const MS_GRID_ROW_SPAN: &str = "ms-grid-row-span";
	pub const MS_GRID_ROWS: &str = "ms-grid-rows";
	pub const MS_HIGH_CONTRAST_ADJUST: &str = "ms-high-contrast-adjust";
	pub const MS_HYPHENATE_LIMIT_CHARS: &str = "ms-hyphenate-limit-chars";
	pub const MS_HYPHENATE_LIMIT_LINES: &str = "ms-hyphenate-limit-lines";
	pub const MS_HYPHENATE_LIMIT_ZONE: &str = "ms-hyphenate-limit-zone";
	pub const MS_HYPHENS: &str = "ms-hyphens";
	pub const MS_IME_ALIGN: &str = "ms-ime-align";
	pub const MS_OVERFLOW_STYLE: &str = "ms-overflow-style";
	pub const MS_SCROLL_CHAINING: &str = "ms-scroll-chaining";
	pub const MS_SCROLL_LIMIT: &str = "ms-scroll-limit";
	pub const MS_SCROLL_LIMIT_X_MAX: &str = "ms-scroll-limit-x_max";
	pub const MS_SCROLL_LIMIT_X_MIN: &str = "ms-scroll-limit-x_min";
	pub const MS_SCROLL_LIMIT_Y_MAX: &str = "ms-scroll-limit-y_max";
	pub const MS_SCROLL_LIMIT_Y_MIN: &str = "ms-scroll-limit-y_min";
	pub const MS_SCROLL_RAILS: &str = "ms-scroll-rails";
	pub const MS_SCROLL_SNAP_POINTS_X: &str = "ms-scroll-snap-points-x";
	pub const MS_SCROLL_SNAP_POINTS_Y: &str = "ms-scroll-snap-points-y";
	pub const MS_SCROLL_SNAP_TYPE: &str = "ms-scroll-snap-type";
	pub const MS_SCROLL_SNAP_X: &str = "ms-scroll-snap-x";
	pub const MS_SCROLL_SNAP_Y: &str = "ms-scroll-snap-y";
	pub const MS_SCROLL_TRANSLATION: &str = "ms-scroll-translation";
	pub const MS_TEXT_COMBINE_HORIZONTAL: &str = "ms-text-combine-horizontal";
	pub const MS_TEXT_SIZE_ADJUST: &str = "ms-text-size-adjust";
	pub const MS_TOUCH_ACTION: &str = "ms-touch-action";
	pub const MS_TOUCH_SELECT: &str = "ms-touch-select";
	pub const MS_USER_SELECT: &str = "ms-user-select";
	pub const MS_WRAP_FLOW: &str = "ms-wrap-flow";
	pub const MS_WRAP_MARGIN: &str = "ms-wrap-margin";
	pub const MS_WRAP_THROUGH: &str = "ms-wrap-through";
	pub const OBJECT_FIT: &str = "object-fit";
	pub const OBJECT_POSITION: &str = "object-position";
	pub const OPACITY: &str = "opacity";
	pub const ORDER: &str = "order";
	pub const ORPHANS: &str = "orphans";
	pub const OUTLINE: &str = "outline";
	pub const OUTLINE_COLOR: &str = "outline-color";
	pub const OUTLINE_OFFSET: &str = "outline-offset";
	pub const OUTLINE_STYLE: &str = "outline-style";
	pub const OUTLINE_WIDTH: &str = "outline-width";
	pub const OVERFLOW: &str = "overflow";
	pub const OVERFLOW_ANCHOR: &str = "overflow-anchor";
	pub const OVERFLOW_WRAP: &str = "overflow-wrap";
	pub const OVERFLOW_X: &str = "overflow-x";
	pub const OVERFLOW_Y: &str = "overflow-y";
	pub const PADDING: &str = "padding";
	pub const PADDING_BLOCK_END: &str = "padding-block-end";
	pub const PADDING_BLOCK_START: &str = "padding-block-start";
	pub const PADDING_BOTTOM: &str = "padding-bottom";
	pub const PADDING_INLINE_END: &str = "padding-inline-end";
	pub const PADDING_INLINE_START: &str = "padding-inline-start";
	pub const PADDING_LEFT: &str = "padding-left";
	pub const PADDING_RIGHT: &str = "padding-right";
	pub const PADDING_TOP: &str = "padding-top";
	pub const PAGE_BREAK_AFTER: &str = "page-break-after";
	pub const PAGE_BREAK_BEFORE: &str = "page-break-before";
	pub const PAGE_BREAK_INSIDE: &str = "page-break-inside";
	pub const PAINT_ORDER: &str = "paint-order";
	pub const PARENT_RULE: &str = "parent-rule";
	pub const PEN_ACTION: &str = "pen-action";
	pub const PERSPECTIVE: &str = "perspective";
	pub const PERSPECTIVE_ORIGIN: &str = "perspective-origin";
	pub const PLACE_CONTENT: &str = "place-content";
	pub const PLACE_ITEMS: &str = "place-items";
	pub const PLACE_SELF: &str = "place-self";
	pub const POINTER_EVENTS: &str = "pointer-events";
	pub const POSITION: &str = "position";
	pub const QUOTES: &str = "quotes";
	pub const RESIZE: &str = "resize";
	pub const RIGHT: &str = "right";
	pub const ROTATE: &str = "rotate";
	pub const ROW_GAP: &str = "row-gap";
	pub const RUBY_ALIGN: &str = "ruby-align";
	pub const RUBY_OVERHANG: &str = "ruby-overhang";
	pub const RUBY_POSITION: &str = "ruby-position";
	pub const SCALE: &str = "scale";
	pub const SCROLL_BEHAVIOR: &str = "scroll-behavior";
	pub const SHAPE_RENDERING: &str = "shape-rendering";
	pub const STOP_COLOR: &str = "stop-color";
	pub const STOP_OPACITY: &str = "stop-opacity";
	pub const STROKE: &str = "stroke";
	pub const STROKE_DASHARRAY: &str = "stroke-dasharray";
	pub const STROKE_DASHOFFSET: &str = "stroke-dashoffset";
	pub const STROKE_LINECAP: &str = "stroke-linecap";
	pub const STROKE_LINEJOIN: &str = "stroke-linejoin";
	pub const STROKE_MITERLIMIT: &str = "stroke-miterlimit";
	pub const STROKE_OPACITY: &str = "stroke-opacity";
	pub const STROKE_WIDTH: &str = "stroke-width";
	pub const TAB_SIZE: &str = "tab-size";
	pub const TABLE_LAYOUT: &str = "table-layout";
	pub const TEXT_ALIGN: &str = "text-align";
	pub const TEXT_ALIGN_LAST: &str = "text-align-last";
	pub const TEXT_ANCHOR: &str = "text-anchor";
	pub const TEXT_COMBINE_UPRIGHT: &str = "text-combine-upright";
	pub const TEXT_DECORATION: &str = "text-decoration";
	pub const TEXT_DECORATION_COLOR: &str = "text-decoration-color";
	pub const TEXT_DECORATION_LINE: &str = "text-decoration-line";
	pub const TEXT_DECORATION_STYLE: &str = "text-decoration-style";
	pub const TEXT_EMPHASIS: &str = "text-emphasis";
	pub const TEXT_EMPHASIS_COLOR: &str = "text-emphasis-color";
	pub const TEXT_EMPHASIS_POSITION: &str = "text-emphasis-position";
	pub const TEXT_EMPHASIS_STYLE: &str = "text-emphasis-style";
	pub const TEXT_INDENT: &str = "text-indent";
	pub const TEXT_JUSTIFY: &str = "text-justify";
	pub const TEXT_KASHIDA: &str = "text-kashida";
	pub const TEXT_KASHIDA_SPACE: &str = "text-kashida-space";
	pub const TEXT_ORIENTATION: &str = "text-orientation";
	pub const TEXT_OVERFLOW: &str = "text-overflow";
	pub const TEXT_RENDERING: &str = "text-rendering";
	pub const TEXT_SHADOW: &str = "text-shadow";
	pub const TEXT_TRANSFORM: &str = "text-transform";
	pub const TEXT_UNDERLINE_POSITION: &str = "text-underline-position";
	pub const TOP: &str = "top";
	pub const TOUCH_ACTION: &str = "touch-action";
	pub const TRANSFORM: &str = "transform";
	pub const TRANSFORM_BOX: &str = "transform-box";
	pub const TRANSFORM_ORIGIN: &str = "transform-origin";
	pub const TRANSFORM_STYLE: &str = "transform-style";
	pub const TRANSITION: &str = "transition";
	pub const TRANSITION_DELAY: &str = "transition-delay";
	pub const TRANSITION_DURATION: &str = "transition-duration";
	pub const TRANSITION_PROPERTY: &str = "transition-property";
	pub const TRANSITION_TIMING_FUNCTION: &str = "transition-timing-function";
	pub const TRANSLATE: &str = "translate";
	pub const UNICODE_BIDI: &str = "unicode-bidi";
	pub const USER_SELECT: &str = "user-select";
	pub const VERTICAL_ALIGN: &str = "vertical-align";
	pub const VISIBILITY: &str = "visibility";
	pub const WEBKIT_ALIGN_CONTENT: &str = "webkit-align-content";
	pub const WEBKIT_ALIGN_ITEMS: &str = "webkit-align-items";
	pub const WEBKIT_ALIGN_SELF: &str = "webkit-align-self";
	pub const WEBKIT_ANIMATION: &str = "webkit-animation";
	pub const WEBKIT_ANIMATION_DELAY: &str = "webkit-animation-delay";
	pub const WEBKIT_ANIMATION_DIRECTION: &str = "webkit-animation-direction";
	pub const WEBKIT_ANIMATION_DURATION: &str = "webkit-animation-duration";
	pub const WEBKIT_ANIMATION_FILL_MODE: &str = "webkit-animation-fill-mode";
	pub const WEBKIT_ANIMATION_ITERATION_COUNT: &str = "webkit-animation-iteration-count";
	pub const WEBKIT_ANIMATION_NAME: &str = "webkit-animation-name";
	pub const WEBKIT_ANIMATION_PLAY_STATE: &str = "webkit-animation-play-state";
	pub const WEBKIT_ANIMATION_TIMING_FUNCTION: &str = "webkit-animation-timing-function";
	pub const WEBKIT_APPEARANCE: &str = "webkit-appearance";
	pub const WEBKIT_BACKFACE_VISIBILITY: &str = "webkit-backface-visibility";
	pub const WEBKIT_BACKGROUND_CLIP: &str = "webkit-background-clip";
	pub const WEBKIT_BACKGROUND_ORIGIN: &str = "webkit-background-origin";
	pub const WEBKIT_BACKGROUND_SIZE: &str = "webkit-background-size";
	pub const WEBKIT_BORDER_BOTTOM_LEFT_RADIUS: &str = "webkit-border-bottom-left-radius";
	pub const WEBKIT_BORDER_BOTTOM_RIGHT_RADIUS: &str = "webkit-border-bottom-right-radius";
	pub const WEBKIT_BORDER_IMAGE: &str = "webkit-border-image";
	pub const WEBKIT_BORDER_RADIUS: &str = "webkit-border-radius";
	pub const WEBKIT_BORDER_TOP_LEFT_RADIUS: &str = "webkit-border-top-left-radius";
	pub const WEBKIT_BORDER_TOP_RIGHT_RADIUS: &str = "webkit-border-top-right-radius";
	pub const WEBKIT_BOX_ALIGN: &str = "webkit-box-align";
	pub const WEBKIT_BOX_DIRECTION: &str = "webkit-box-direction";
	pub const WEBKIT_BOX_FLEX: &str = "webkit-box-flex";
	pub const WEBKIT_BOX_ORDINAL_GROUP: &str = "webkit-box-ordinal-group";
	pub const WEBKIT_BOX_ORIENT: &str = "webkit-box-orient";
	pub const WEBKIT_BOX_PACK: &str = "webkit-box-pack";
	pub const WEBKIT_BOX_SHADOW: &str = "webkit-box-shadow";
	pub const WEBKIT_BOX_SIZING: &str = "webkit-box-sizing";
	pub const WEBKIT_COLUMN_BREAK_AFTER: &str = "webkit-column-break-after";
	pub const WEBKIT_COLUMN_BREAK_BEFORE: &str = "webkit-column-break-before";
	pub const WEBKIT_COLUMN_BREAK_INSIDE: &str = "webkit-column-break-inside";
	pub const WEBKIT_COLUMN_COUNT: &str = "webkit-column-count";
	pub const WEBKIT_COLUMN_GAP: &str = "webkit-column-gap";
	pub const WEBKIT_COLUMN_RULE: &str = "webkit-column-rule";
	pub const WEBKIT_COLUMN_RULE_COLOR: &str = "webkit-column-rule-color";
	pub const WEBKIT_COLUMN_RULE_STYLE: &str = "webkit-column-rule-style";
	pub const WEBKIT_COLUMN_RULE_WIDTH: &str = "webkit-column-rule-width";
	pub const WEBKIT_COLUMN_SPAN: &str = "webkit-column-span";
	pub const WEBKIT_COLUMN_WIDTH: &str = "webkit-column-width";
	pub const WEBKIT_COLUMNS: &str = "webkit-columns";
	pub const WEBKIT_FILTER: &str = "webkit-filter";
	pub const WEBKIT_FLEX: &str = "webkit-flex";
	pub const WEBKIT_FLEX_BASIS: &str = "webkit-flex-basis";
	pub const WEBKIT_FLEX_DIRECTION: &str = "webkit-flex-direction";
	pub const WEBKIT_FLEX_FLOW: &str = "webkit-flex-flow";
	pub const WEBKIT_FLEX_GROW: &str = "webkit-flex-grow";
	pub const WEBKIT_FLEX_SHRINK: &str = "webkit-flex-shrink";
	pub const WEBKIT_FLEX_WRAP: &str = "webkit-flex-wrap";
	pub const WEBKIT_JUSTIFY_CONTENT: &str = "webkit-justify-content";
	pub const WEBKIT_LINE_CLAMP: &str = "webkit-line-clamp";
	pub const WEBKIT_MASK: &str = "webkit-mask";
	pub const WEBKIT_MASK_BOX_IMAGE: &str = "webkit-mask-box-image";
	pub const WEBKIT_MASK_BOX_IMAGE_OUTSET: &str = "webkit-mask-box-image-outset";
	pub const WEBKIT_MASK_BOX_IMAGE_REPEAT: &str = "webkit-mask-box-image-repeat";
	pub const WEBKIT_MASK_BOX_IMAGE_SLICE: &str = "webkit-mask-box-image-slice";
	pub const WEBKIT_MASK_BOX_IMAGE_SOURCE: &str = "webkit-mask-box-image-source";
	pub const WEBKIT_MASK_BOX_IMAGE_WIDTH: &str = "webkit-mask-box-image-width";
	pub const WEBKIT_MASK_CLIP: &str = "webkit-mask-clip";
	pub const WEBKIT_MASK_COMPOSITE: &str = "webkit-mask-composite";
	pub const WEBKIT_MASK_IMAGE: &str = "webkit-mask-image";
	pub const WEBKIT_MASK_ORIGIN: &str = "webkit-mask-origin";
	pub const WEBKIT_MASK_POSITION: &str = "webkit-mask-position";
	pub const WEBKIT_MASK_REPEAT: &str = "webkit-mask-repeat";
	pub const WEBKIT_MASK_SIZE: &str = "webkit-mask-size";
	pub const WEBKIT_ORDER: &str = "webkit-order";
	pub const WEBKIT_PERSPECTIVE: &str = "webkit-perspective";
	pub const WEBKIT_PERSPECTIVE_ORIGIN: &str = "webkit-perspective-origin";
	pub const WEBKIT_TAP_HIGHLIGHT_COLOR: &str = "webkit-tap-highlight-color";
	pub const WEBKIT_TEXT_FILL_COLOR: &str = "webkit-text-fill-color";
	pub const WEBKIT_TEXT_SIZE_ADJUST: &str = "webkit-text-size-adjust";
	pub const WEBKIT_TEXT_STROKE: &str = "webkit-text-stroke";
	pub const WEBKIT_TEXT_STROKE_COLOR: &str = "webkit-text-stroke-color";
	pub const WEBKIT_TEXT_STROKE_WIDTH: &str = "webkit-text-stroke-width";
	pub const WEBKIT_TRANSFORM: &str = "webkit-transform";
	pub const WEBKIT_TRANSFORM_ORIGIN: &str = "webkit-transform-origin";
	pub const WEBKIT_TRANSFORM_STYLE: &str = "webkit-transform-style";
	pub const WEBKIT_TRANSITION: &str = "webkit-transition";
	pub const WEBKIT_TRANSITION_DELAY: &str = "webkit-transition-delay";
	pub const WEBKIT_TRANSITION_DURATION: &str = "webkit-transition-duration";
	pub const WEBKIT_TRANSITION_PROPERTY: &str = "webkit-transition-property";
	pub const WEBKIT_TRANSITION_TIMING_FUNCTION: &str = "webkit-transition-timing-function";
	pub const WEBKIT_USER_MODIFY: &str = "webkit-user-modify";
	pub const WEBKIT_USER_SELECT: &str = "webkit-user-select";
	pub const WEBKIT_WRITING_MODE: &str = "webkit-writing-mode";
	pub const WHITE_SPACE: &str = "white-space";
	pub const WIDOWS: &str = "widows";
	pub const WIDTH: &str = "width";
	pub const WILL_CHANGE: &str = "will-change";
	pub const WORD_BREAK: &str = "word-break";
	pub const WORD_SPACING: &str = "word-spacing";
	pub const WORD_WRAP: &str = "word-wrap";
	pub const WRITING_MODE: &str = "writing-mode";
	pub const Z_INDEX: &str = "z-index";
	pub const ZOOM: &str = "zoom";
}
