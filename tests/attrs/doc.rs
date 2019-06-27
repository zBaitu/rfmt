#![doc(html_favicon_url = "https://example.com/favicon.ico")]
//!aaaaaaaaaaaaaa
//!bbbbbbbbbbbbbb
/*!cccccccccc
 * ddddddddd
 !*/
//!

/// aaaaaaaaaaa
/// bbbbbbbbbb
/** cccccccccc **/
const a: bool = true;

// abcde
/* abcde */ const b: bool = true;
/* aaaaaaa
 *
 * aaaaaa
 */

#[doc = " This is a doc comment."]
const c: bool = true;
