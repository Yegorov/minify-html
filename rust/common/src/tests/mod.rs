fn eval_with_cfg(src: &'static [u8], expected: &'static [u8], cfg: &super::Cfg) {
    let mut code = src.to_vec();
    let min = super::minify(&mut code, cfg);
    assert_eq!(
        std::str::from_utf8(&min).unwrap(),
        std::str::from_utf8(expected).unwrap(),
    );
}

fn eval(src: &'static [u8], expected: &'static [u8]) {
    eval_with_cfg(src, expected, &super::Cfg::new());
}

fn eval_with_keep_html_head(src: &'static [u8], expected: &'static [u8]) -> () {
    let mut cfg = super::Cfg::new();
    cfg.keep_html_and_head_opening_tags = true;
    eval_with_cfg(src, expected, &cfg);
}

#[cfg(feature = "js-esbuild")]
fn eval_with_js_min(src: &'static [u8], expected: &'static [u8]) -> () {
    let mut cfg = super::Cfg::new();
    cfg.minify_js = true;
    eval_with_cfg(src, expected, &cfg);
}

#[cfg(feature = "js-esbuild")]
fn eval_with_css_min(src: &'static [u8], expected: &'static [u8]) -> () {
    let mut cfg = super::Cfg::new();
    cfg.minify_css = true;
    eval_with_cfg(src, expected, &cfg);
}

#[test]
fn test_collapse_whitespace() {
    eval(b"<a>   \n&#32;   </a>", b"<a> </a>");
    // Tag names should be case insensitive.
    eval(b"<A>   \n&#32;   </a>", b"<a> </a>");
    eval(b"<a>   \n&#32;   </A>", b"<a> </a>");
}

#[test]
fn test_collapse_and_trim_whitespace() {
    eval(b"<label>   \n&#32;   </label>", b"<label></label>");
    eval(b"<label>   \n&#32;a   </label>", b"<label>a</label>");
    eval(b"<label>   \n&#32;a   b   </label>", b"<label>a b</label>");
    // Tag names should be case insensitive.
    eval(b"<lAbEL>   \n&#32;a   b   </LABel>", b"<label>a b</label>");
}

#[test]
fn test_collapse_destroy_whole_and_trim_whitespace() {
    eval(b"<ul>   \n&#32;   </ul>", b"<ul></ul>");
    eval(b"<ul>   \n&#32;a   </ul>", b"<ul>a</ul>");
    eval(b"<ul>   \n&#32;a   b   </ul>", b"<ul>a b</ul>");
    eval(
        b"<ul>   \n&#32;a<pre></pre>   <pre></pre>b   </ul>",
        b"<ul>a<pre></pre><pre></pre>b</ul>",
    );
    // Tag names should be case insensitive.
    eval(b"<uL>   \n&#32;a   b   </UL>", b"<ul>a b</ul>");
}

#[test]
fn test_no_whitespace_minification() {
    eval(b"<pre>   \n&#32; \t   </pre>", b"<pre>   \n  \t   </pre>");
    eval(
        b"<textarea>   \n&#32; \t   </textarea>",
        b"<textarea>   \n  \t   </textarea>",
    );
    // Tag names should be case insensitive.
    eval(b"<pRe>   \n&#32; \t   </PRE>", b"<pre>   \n  \t   </pre>");
    eval(
        b"<pre>  <span>  1    2   </span>  </pre>",
        b"<pre>  <span>  1    2   </span>  </pre>",
    );
    eval(
        b"<pre>  <span>  1 <pre>\n</pre>    2   </span>  </pre>",
        b"<pre>  <span>  1 <pre>\n</pre>    2   </span>  </pre>",
    );
    eval(
        b"<div>  <pre>  <span>  1 <pre>\n</pre>    2   </span>  </pre>  </div>",
        b"<div><pre>  <span>  1 <pre>\n</pre>    2   </span>  </pre></div>",
    );
    eval(
        br#"<pre><code>fn main() {
  println!("Hello, world!");
  <span>loop {
    println!("Hello, world!");
  }</span>
}
</code></pre>"#,
        br#"<pre><code>fn main() {
  println!("Hello, world!");
  <span>loop {
    println!("Hello, world!");
  }</span>
}
</code></pre>"#,
    );
}

#[test]
fn test_parsing_extra_head_tag() {
    // Extra `<head>` in `<label>` should be dropped, so whitespace around `<head>` should be joined and therefore trimmed due to `<label>` whitespace rules.
    eval_with_keep_html_head(
        b"<html><head><meta><head><link><head><body><label>  <pre> </pre> <head>  </label>",
        b"<html><head><meta><link><body><label><pre> </pre></label>",
    );
    // Same as above except it's a `</head>`, which should get reinterpreted as a `<head>`.
    eval_with_keep_html_head(
        b"<html><head><meta><head><link><head><body><label>  <pre> </pre> </head>  </label>",
        b"<html><head><meta><link><body><label><pre> </pre></label>",
    );
    // `<head>` gets implicitly closed by `<body>`, so any following `</head>` should be ignored. (They should be anyway, since `</head>` would not be a valid closing tag.)
    eval_with_keep_html_head(
        b"<html><head><body><label> </head> </label>",
        b"<html><head><body><label></label>",
    );
}

#[test]
fn test_parsing_omitted_closing_tag() {
    eval_with_keep_html_head(b"<html>", b"<html>");
    eval_with_keep_html_head(b" <html>\n", b"<html>");
    eval_with_keep_html_head(b" <!doctype html> <html>\n", b"<!doctype html><html>");
    eval_with_keep_html_head(
        b"<!doctype html><html><div> <p>Foo</div></html>",
        b"<!doctype html><html><div><p>Foo</div>",
    );
}

#[test]
fn test_self_closing_svg_tag_whitespace_removal() {
    eval(b"<svg><path d=a /></svg>", b"<svg><path d=a /></svg>");
    eval(b"<svg><path d=a/ /></svg>", b"<svg><path d=a/ /></svg>");
    eval(b"<svg><path d=\"a/\" /></svg>", b"<svg><path d=a/ /></svg>");
    eval(b"<svg><path d=\"a/\"/></svg>", b"<svg><path d=a/ /></svg>");
    eval(b"<svg><path d='a/' /></svg>", b"<svg><path d=a/ /></svg>");
    eval(b"<svg><path d='a/'/></svg>", b"<svg><path d=a/ /></svg>");
}

#[test]
fn test_parsing_with_omitted_tags() {
    eval_with_keep_html_head(b"<ul><li>1<li>2<li>3</ul>", b"<ul><li>1<li>2<li>3</ul>");
    eval_with_keep_html_head(b"<rt>", b"<rt>");
    eval_with_keep_html_head(b"<rt><rp>1</rp><div></div>", b"<rt><rp>1</rp><div></div>");
    eval_with_keep_html_head(b"<div><rt></div>", b"<div><rt></div>");
    eval_with_keep_html_head(b"<html><head><body>", b"<html><head><body>");
    eval_with_keep_html_head(b"<html><head><body>", b"<html><head><body>");
    // Tag names should be case insensitive.
    eval_with_keep_html_head(b"<rt>", b"<rt>");
}

#[test]
fn test_unmatched_closing_tag() {
    eval_with_keep_html_head(b"Hello</p>Goodbye", b"Hello<p>Goodbye");
    eval_with_keep_html_head(b"Hello<br></br>Goodbye", b"Hello<br>Goodbye");
    eval_with_keep_html_head(b"<div>Hello</p>Goodbye", b"<div>Hello<p>Goodbye");
    eval_with_keep_html_head(b"<ul><li>a</p>", b"<ul><li>a<p>");
    eval_with_keep_html_head(b"<ul><li><rt>a</p>", b"<ul><li><rt>a<p>");
    eval_with_keep_html_head(
        b"<html><head><body><ul><li><rt>a</p>",
        b"<html><head><body><ul><li><rt>a<p>",
    );
}

#[test]
fn test_removal_of_html_and_head_opening_tags() {
    // Even though `<head>` is dropped, it's still parsed, so its content is still subject to `<head>` whitespace minification rules.
    eval(
        b"<!DOCTYPE html><html><head>  <meta> <body>",
        b"<!DOCTYPE html><meta><body>",
    );
    // The tag should not be dropped if it has attributes.
    eval(
        b"<!DOCTYPE html><html lang=en><head>  <meta> <body>",
        b"<!DOCTYPE html><html lang=en><meta><body>",
    );
}

#[test]
fn test_removal_of_optional_tags() {
    eval_with_keep_html_head(
        b"<ul><li>1</li><li>2</li><li>3</li></ul>",
        b"<ul><li>1<li>2<li>3</ul>",
    );
    eval_with_keep_html_head(b"<rt></rt>", b"<rt>");
    eval_with_keep_html_head(
        b"<rt></rt><rp>1</rp><div></div>",
        b"<rt><rp>1</rp><div></div>",
    );
    eval_with_keep_html_head(b"<div><rt></rt></div>", b"<div><rt></div>");
    eval_with_keep_html_head(
        br#"
        <html>
            <head>
            </head>

            <body>
            </body>
        </html>
    "#,
        b"<html><head><body>",
    );
    // Tag names should be case insensitive.
    eval_with_keep_html_head(b"<RT></rt>", b"<rt>");
}

#[test]
fn test_removal_of_optional_closing_p_tag() {
    eval(b"<p></p><address></address>", b"<p><address></address>");
    eval(b"<p></p>", b"<p>");
    eval(b"<map><p></p></map>", b"<map><p></p></map>");
    eval(
        b"<map><p></p><address></address></map>",
        b"<map><p><address></address></map>",
    );
}

#[test]
fn test_attr_double_quoted_value_minification() {
    eval(b"<a b=\" hello \"></a>", b"<a b=\" hello \"></a>");
    eval(b"<a b=' hello '></a>", b"<a b=\" hello \"></a>");
    eval(br#"<a b="/>aaaa"></a>"#, br#"<a b="/>aaaa"></a>"#);
    eval(br#"<a b="</a>a"></a>"#, br#"<a b="</a>a"></a>"#);
    eval(b"<a b=&#x20;hello&#x20;></a>", b"<a b=\" hello \"></a>");
    eval(b"<a b=&#x20hello&#x20></a>", b"<a b=\" hello \"></a>");
}

#[test]
fn test_attr_single_quoted_value_minification() {
    eval(b"<a b=\"&quot;hello\"></a>", b"<a b='\"hello'></a>");
    eval(b"<a b='\"hello'></a>", b"<a b='\"hello'></a>");
    eval(b"<a b='/>a'></a>", b"<a b=\"/>a\"></a>");
    eval(
        b"<a b=&#x20;he&quot;llo&#x20;></a>",
        b"<a b=' he\"llo '></a>",
    );
}

#[test]
fn test_attr_unquoted_value_minification() {
    eval(b"<a b==></a>", b"<a b==></a>");
    eval(b"<a b=`'\"<<==/`/></a>", b"<a b=`'\"<<==/`/></a>");
    eval(b"<a b=\"hello\"></a>", b"<a b=hello></a>");
    eval(b"<a b='hello'></a>", b"<a b=hello></a>");
    eval(b"<a b=/&gt></a>", br#"<a b="/>"></a>"#);
    eval(b"<a b=/&gt&lt;a></a>", br#"<a b="/><a"></a>"#);
    eval(b"<a b=hello></a>", b"<a b=hello></a>");
}

#[test]
fn test_attr_whatwg_unquoted_value_minification() {
    let mut cfg = super::Cfg::new();
    cfg.ensure_spec_compliant_unquoted_attribute_values = true;
    eval_with_cfg(b"<a b==></a>", br#"<a b="="></a>"#, &cfg);
    eval_with_cfg(
        br#"<a b=`'"<<==/`/></a>"#,
        br#"<a b="`'&#34<<==/`/"></a>"#,
        &cfg,
    );
}

#[test]
fn test_class_attr_value_minification() {
    eval(b"<a class=&#x20;c></a>", b"<a class=c></a>");
    eval(
        b"<a class=&#x20;c&#x20&#x20;d&#x20></a>",
        b"<a class=\"c d\"></a>",
    );
    eval(b"<a class=&#x20&#x20&#x20;&#x20></a>", b"<a></a>");
    eval(b"<a class=\"  c\n \n  \"></a>", b"<a class=c></a>");
    eval(b"<a class=\"  c\n \nd  \"></a>", b"<a class=\"c d\"></a>");
    eval(b"<a class=\"  \n \n  \"></a>", b"<a></a>");
    eval(b"<a class='  c\n \n  '></a>", b"<a class=c></a>");
    eval(b"<a class='  c\n \nd  '></a>", b"<a class=\"c d\"></a>");
    eval(b"<a class='  \n \n  '></a>", b"<a></a>");
    // Attribute names should be case insensitive.
    eval(b"<a CLasS='  \n \n  '></a>", b"<a></a>");
}

#[test]
fn test_d_attr_value_minification() {
    eval(b"<svg><path d=&#x20;c /></svg>", b"<svg><path d=c /></svg>");
    eval(
        b"<svg><path d=&#x20;c&#x20&#x20;d&#x20 /></svg>",
        b"<svg><path d=\"c d\"/></svg>",
    );
    eval(
        b"<svg><path d=&#x20;&#x20&#x20&#x20 /></svg>",
        b"<svg><path/></svg>",
    );
    eval(
        b"<svg><path d=\"  c\n \n  \" /></svg>",
        b"<svg><path d=c /></svg>",
    );
    eval(
        b"<svg><path d=\"  c\n \nd  \" /></svg>",
        b"<svg><path d=\"c d\"/></svg>",
    );
    eval(
        b"<svg><path d=\"  \n \n  \" /></svg>",
        b"<svg><path/></svg>",
    );
    eval(
        b"<svg><path d='  c\n \n  ' /></svg>",
        b"<svg><path d=c /></svg>",
    );
    eval(
        b"<svg><path d='  c\n \nd  ' /></svg>",
        b"<svg><path d=\"c d\"/></svg>",
    );
    eval(b"<svg><path d='  \n \n  ' /></svg>", b"<svg><path/></svg>");
    // Attribute names should be case insensitive.
    eval(b"<svg><path D='  \n \n  ' /></svg>", b"<svg><path/></svg>");
}

#[test]
fn test_boolean_attr_value_removal() {
    eval(b"<div hidden=\"true\"></div>", b"<div hidden></div>");
    eval(b"<div hidden=\"false\"></div>", b"<div hidden></div>");
    eval(b"<div hidden=\"1\"></div>", b"<div hidden></div>");
    eval(b"<div hidden=\"0\"></div>", b"<div hidden></div>");
    eval(b"<div hidden=\"abc\"></div>", b"<div hidden></div>");
    eval(b"<div hidden=\"\"></div>", b"<div hidden></div>");
    eval(b"<div hidden></div>", b"<div hidden></div>");
    // Attribute names should be case insensitive.
    eval(b"<div HIDden=\"true\"></div>", b"<div hidden></div>");
}

#[test]
fn test_empty_attr_removal() {
    eval(b"<div lang=\"  \"></div>", b"<div lang=\"  \"></div>");
    eval(b"<div lang=\"\"></div>", b"<div></div>");
    eval(b"<div lang=''></div>", b"<div></div>");
    eval(b"<div lang=></div>", b"<div></div>");
    eval(b"<div lang></div>", b"<div></div>");
}

#[test]
fn test_default_attr_value_removal() {
    eval(b"<a target=\"_self\"></a>", b"<a></a>");
    eval(b"<a target='_self'></a>", b"<a></a>");
    eval(b"<a target=_self></a>", b"<a></a>");
    // Attribute names should be case insensitive.
    eval(b"<a taRGET='_self'></a>", b"<a></a>");
}

#[test]
fn test_script_type_attr_value_removal() {
    eval(
        b"<script type=\"application/ecmascript\"></script>",
        b"<script></script>",
    );
    eval(
        b"<script type=\"application/javascript\"></script>",
        b"<script></script>",
    );
    eval(
        b"<script type=\"text/jscript\"></script>",
        b"<script></script>",
    );
    eval(
        b"<script type=\"text/plain\"></script>",
        b"<script type=text/plain></script>",
    );
    // Tag and attribute names should be case insensitive.
    eval(
        b"<SCRipt TYPE=\"application/ecmascript\"></SCrIPT>",
        b"<script></script>",
    );
}

#[test]
fn test_empty_attr_value_removal() {
    eval(b"<div a=\"  \"></div>", b"<div a=\"  \"></div>");
    eval(b"<div a=\"\"></div>", b"<div a></div>");
    eval(b"<div a=''></div>", b"<div a></div>");
    eval(b"<div a=></div>", b"<div a></div>");
    eval(b"<div a></div>", b"<div a></div>");
}

#[test]
fn test_space_between_attrs_minification() {
    eval(
        b"<div a=\" \" b=\" \"></div>",
        b"<div a=\" \"b=\" \"></div>",
    );
    eval(b"<div a=' ' b=\" \"></div>", b"<div a=\" \"b=\" \"></div>");
    eval(
        b"<div a=&#x20 b=\" \"></div>",
        b"<div a=\" \"b=\" \"></div>",
    );
    eval(b"<div a=\"1\" b=\" \"></div>", b"<div a=1 b=\" \"></div>");
    eval(b"<div a='1' b=\" \"></div>", b"<div a=1 b=\" \"></div>");
    eval(b"<div a=\"a\"b=\"b\"></div>", b"<div a=a b=b></div>");
}

#[test]
fn test_hexadecimal_entity_decoding() {
    eval(b"&#x2E", b".");
    eval(b"&#x2F", b"/");
    eval(b"&#x2f", b"/");
    eval(b"&#x00", b"\0");
    eval(b"&#x30", b"0");
    eval(b"&#x0030", b"0");
    eval(b"&#x000000000000000000000000000000000000000000030", b"0");
    eval(b"&#x30;", b"0");
    eval(b"&#x0030;", b"0");
    eval(b"&#x000000000000000000000000000000000000000000030;", b"0");
    eval(b"&#x1151;", b"\xe1\x85\x91");
    eval(b"&#x11FFFF;", b"\xef\xbf\xbd");
    eval(
        b"&#xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;",
        b"\xef\xbf\xbd",
    );
}

#[test]
fn test_decimal_entity_decoding() {
    eval(b"&#48", b"0");
    eval(b"&#0048", b"0");
    eval(b"&#000000000000000000000000000000000000000000048", b"0");
    eval(b"&#48;", b"0");
    eval(b"&#0048;", b"0");
    eval(b"&#000000000000000000000000000000000000000000048;", b"0");
    eval(b"&#4433;", b"\xe1\x85\x91");
    eval(b"&#1114112;", b"\xef\xbf\xbd");
    eval(
        b"&#999999999999999999999999999999999999999999999;",
        b"\xef\xbf\xbd",
    );
}

#[test]
fn test_named_entity_decoding() {
    eval(b"&gt", b">");
    eval(b"&gt;", b">");
    eval(b"&amp", b"&");
    eval(b"&amp;", b"&");
    eval(b"&xxxyyyzzz", b"&xxxyyyzzz");
    eval(b"&ampere", b"&ere");
    eval(b"They & Co.", b"They & Co.");
    eval(b"if (this && that)", b"if (this && that)");
    // These entities decode to longer UTF-8 sequences, so we keep them encoded.
    eval(b"&nLt;", b"&nLt;");
    eval(b"&nLt;abc", b"&nLt;abc");
    eval(b"&nGt;", b"&nGt;");

    // Named entities not ending with ';' in attr values are not decoded if immediately
    // followed by an alphanumeric or `=` character. (See parser for more details.)
    eval(
        br#"<a href="exam ple?&gta=5"></a>"#,
        br#"<a href="exam ple?&gta=5"></a>"#,
    );
    eval(
        br#"<a href="exam ple?&gt=5"></a>"#,
        br#"<a href="exam ple?&gt=5"></a>"#,
    );
    eval(
        br#"<a href="exam ple?&gt~5"></a>"#,
        br#"<a href="exam ple?>~5"></a>"#,
    );
}

#[test]
fn test_unintentional_entity_prevention() {
    eval(b"&ampamp", b"&ampamp");
    eval(b"&ampamp;", b"&ampamp;");
    eval(b"&amp;amp", b"&ampamp");
    eval(b"&amp;amp;", b"&ampamp;");
    eval(b"&&#97&#109;&#112;;", b"&ampamp;");
    eval(b"&&#97&#109;p;", b"&ampamp;");
    eval(b"&am&#112", b"&ampamp");
    eval(b"&am&#112;", b"&ampamp");
    eval(b"&am&#112&#59", b"&ampamp;");
    eval(b"&am&#112;;", b"&ampamp;");
    eval(b"&am&#112;&#59", b"&ampamp;");
    eval(b"&am&#112;&#59;", b"&ampamp;");

    eval(b"&l&#116", b"&amplt");
    eval(b"&&#108t", b"&amplt");
    eval(b"&&#108t;", b"&amplt;");
    eval(b"&&#108t&#59", b"&amplt;");
    eval(b"&amplt", b"&amplt");
    eval(b"&amplt;", b"&amplt;");

    eval(b"&am&am&#112", b"&am&ampamp");
    eval(b"&am&am&#112&#59", b"&am&ampamp;");

    eval(b"&amp&nLt;", b"&&nLt;");
    eval(b"&am&nLt;", b"&am&nLt;");
    eval(b"&am&nLt;a", b"&am&nLt;a");
    eval(b"&am&nLt", b"&am&nLt");
}

#[test]
fn test_left_chevron_in_content() {
    eval(b"<pre><</pre>", b"<pre><</pre>");
    eval(b"<pre>< </pre>", b"<pre>< </pre>");
    eval(b"<pre> < </pre>", b"<pre> < </pre>");

    eval(b"<pre> &lta </pre>", b"<pre> &LTa </pre>");
    eval(b"<pre> &lt;a </pre>", b"<pre> &LTa </pre>");
    eval(b"<pre> &LTa </pre>", b"<pre> &LTa </pre>");
    eval(b"<pre> &LT;a </pre>", b"<pre> &LTa </pre>");

    eval(b"<pre> &lt? </pre>", b"<pre> &LT? </pre>");
    eval(b"<pre> &lt;? </pre>", b"<pre> &LT? </pre>");
    eval(b"<pre> &LT? </pre>", b"<pre> &LT? </pre>");
    eval(b"<pre> &LT;? </pre>", b"<pre> &LT? </pre>");

    eval(b"<pre> &lt;/ </pre>", b"<pre> &LT/ </pre>");
    eval(b"<pre> &lt;! </pre>", b"<pre> &LT! </pre>");

    eval(b"&LT", b"<");
    eval(b"&LT;", b"<");
    eval(b"&LT;;", b"<;");
    eval(b"&LT;&#59", b"<;");
    eval(b"&LT;&#59;", b"<;");
    eval(b"&lt", b"<");
    eval(b"&lt;", b"<");
    eval(b"&lt;;", b"<;");
    eval(b"&lt;&#59", b"<;");
    eval(b"&lt;&#59;", b"<;");

    eval(b"&LTa", b"&LTa");
    eval(b"&LT;a", b"&LTa");
    eval(b"&LT;a;", b"&LTa;");
    eval(b"&LT;a&#59", b"&LTa;");
    eval(b"&LT;a&#59;", b"&LTa;");
    eval(b"&LT;a;&#59;", b"&LTa;;");

    eval(b"&lt;&#33", b"&LT!");
    eval(b"&lt;&#38", b"<&");
    eval(b"&lt;&#47", b"&LT/");
    eval(b"&lt;&#63", b"&LT?");
    eval(b"&lt;&#64", b"<@");
}

#[test]
fn test_comments_removal() {
    eval(
        b"<pre>a <!-- akd--sj\n <!-- \t\0f--ajk--df->lafj -->  b</pre>",
        b"<pre>a   b</pre>",
    );
    eval(b"&a<!-- akd--sj\n <!-- \t\0f--ajk--df->lafj -->mp", b"&amp");
    eval(
        b"<script><!-- akd--sj\n <!-- \t\0f--ajk--df->lafj --></script>",
        b"<script><!-- akd--sj\n <!-- \t\0f--ajk--df->lafj --></script>",
    );
}

#[test]
fn test_processing_instructions() {
    eval(b"<?php hello??? >>  ?>", b"<?php hello??? >>  ?>");
    eval(b"av<?xml 1.0 ?>g", b"av<?xml 1.0 ?>g");
}

#[cfg(feature = "js-esbuild")]
#[test]
fn test_js_minification() {
    eval_with_js_min(b"<script>let a = 1;</script>", b"<script>let a=1;</script>");
    eval_with_js_min(
        br#"
        <script>let a = 1;</script>
        <script>let b = 2;</script>
    "#,
        b"<script>let a=1;</script><script>let b=2;</script>",
    );
    eval_with_js_min(
        b"<scRIPt type=text/plain>   alert(1.00000);   </scripT>",
        b"<script type=text/plain>   alert(1.00000);   </script>",
    );
    eval_with_js_min(
        br#"
        <script>
            // This is a comment.
            let a = 1;
        </script>
    "#,
        b"<script>let a=1;</script>",
    );
}

#[cfg(feature = "js-esbuild")]
#[test]
fn test_js_minification_unintentional_closing_tag() {
    eval_with_js_min(
        br#"<script>let a = "</" + "script>";</script>"#,
        br#"<script>let a="<\/script>";</script>"#,
    );
    eval_with_js_min(
        br#"<script>let a = "</S" + "cRiPT>";</script>"#,
        br#"<script>let a="<\/ScRiPT>";</script>"#,
    );
    eval_with_js_min(
        br#"<script>let a = "\u003c/script>";</script>"#,
        br#"<script>let a="<\/script>";</script>"#,
    );
    eval_with_js_min(
        br#"<script>let a = "\u003c/scrIPt>";</script>"#,
        br#"<script>let a="<\/scrIPt>";</script>"#,
    );
}

#[cfg(feature = "js-esbuild")]
#[test]
fn test_css_minification() {
    // `<style>` contents.
    eval_with_css_min(
        b"<style>div { color: yellow }</style>",
        b"<style>div{color:#ff0}</style>",
    );
    // `style` attributes.
    eval_with_css_min(
        br#"<div style="color: yellow;"></div>"#,
        br#"<div style=color:#ff0></div>"#,
    );
    // `style` attributes are removed if fully minified away.
    eval_with_css_min(br#"<div style="  /*  */   "></div>"#, br#"<div></div>"#);
}
