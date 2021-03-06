//! An ACON-parsing library
//!
//! This crate contains an ACON-to-tree parser that deserializes text.
//! It can also serialize an ACON tree into text.
//!
//! ACON stands for Awk-Compatible Object Notation.
//! It is used because of its simplicity and versatility.
//!
//! # Examples of Acon #
//!
//! ```notrust
//! key value
//! other-key value
//! and_yet_another_key and some values
//! ```
//! The key is always the first word on the line. The value consists of all remaining words on
//! that line, trimmed by whitespace. Any superfluous whitespace between words is made
//! into a single space. This format makes it very easy to process with awk.
//!
//! # Tables #
//!
//! ```notrust
//! { table
//!   key value
//! }
//! { other-table
//!   key value
//! }
//! key value
//! ```
//!
//! A table is denoted by the first word being a curly opening brace on a line. The name
//! of the table is the second word. If there is no name, the table's name will be empty.
//!
//! # Arrays #
//!
//! ```notrust
//! [ array-name
//!   here is a single value, every line is its own value
//!   this is the second entry
//! ]
//! ```
//!
//! Arrays start when the first word on a line is an opening square bracket.
//! An array has no keys, only values. Arrays are ordered. Empty lines
//! Will become empty elements. In tables empty lines are simply ignored.
//!
//! # Super Delimiter #
//!
//! If you have some deeply nesting structure, or a program that may not finish
//! writing all closing delimiters, you can use '$' as a delimiter. This will
//! close all open tables and arrays.
//!
//! ```notrust
//! { deeply
//!    { nested
//!       [ arrays
//! $ <- we've had enough, anything after the $ on this line is skipped.
//!
//! key value
//! ```
//!
//! # Dot-Pathing #
//!
//! All values can be retrieved using a dot-separated key-path.
//!
//! ```rust
//! use acon::Acon;
//! let input = r#"
//! { table
//!    key value
//!   [ my-array
//!     { subtable
//!       anything goes
//!     }
//!   ]
//! }"#;
//! let result = input.parse::<Acon>().unwrap();
//! assert_eq!(result.path("table.my-array.0.subtable.anything").unwrap().string(), "goes");
//! ```
//!
//! # Escaping #
//!
//! If you want a new-line or explicit whitespace in your value, you need to use escape codes.
//! Dots and whitespaces in keys also require escape codes.
//! Escaping is done by inserting (number), where number is the numeric code point value.
//! This library handles escaping transparently. To escape or unescape is only necessary for
//! other utilities or viewing the data in another way.
//! When using dot-pathing, you still need to explicitly write the parenthesized elements.
//!
//! ```rust
//! use acon::Acon;
//! let input = r#"
//!   key(32)with_space(46)and_dot value(10)with(10)new(10)lines, which is interesting
//! "#;
//! let result = input.parse::<Acon>().unwrap();
//! assert_eq!(result.path("key(32)with_space(46)and_dot").unwrap().string(), "value(10)with(10)new(10)lines, which is interesting");
//! ```
//!
//! # Comments #
//!
//! A line is ignored if the first word is a '#'. If you need this to be the first word
//! on a line, you can use the escape code '(35)'.
//!

#![deny(missing_docs)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", allow(items_after_statements))]
#![feature(test)]

extern crate test;

use std::collections::BTreeMap;
use std::str::FromStr;

/// Vec of Acon values
pub type Array = Vec<Acon>;

/// `BTreeMap` of strings mapped to Acon
pub type Table = BTreeMap<String, Acon>;

/// Enumeration over all variable types in ACON
#[derive(PartialEq, Clone, Debug)]
pub enum Acon {
	/// Array type contains a Vec of Acon
	Array(Array),
	/// String type contains a simple std::string::String
	String(String),
	/// Table consists of a BTreeMap<String, Acon>
	Table(Table),
}

impl Acon {

	/// Assert that this value is an array, else panic
	pub fn array(&self) -> &Array {
		match *self {
			Acon::Array(ref array) => array,
			_ => panic!("Value is not an array"),
		}
	}

	/// Assert that this value is a string, else panic
	pub fn string(&self) -> &String {
		match *self {
			Acon::String(ref string) => string,
			_ => panic!("Value is not a string"),
		}
	}

	/// Assert that this value is a table, else panic
	pub fn table(&self) -> &Table {
		match *self {
			Acon::Table(ref table) => table,
			_ => panic!("Value is not a table"),
		}
	}

	/// Retrieve a reference to an entry via its path
	/// Paths are dot-separated.
	///
	///  ```
	///  use acon::Acon;
	///  let input = r#"
	///  { table
	///    [ array
	///      value
	///  $
	///  "#;
	///  let result = input.parse::<Acon>().unwrap();
	///  assert_eq!(result.path("table.array.0").unwrap().string(), "value");
	///  ```
	///
	pub fn path(&self, path: &str) -> Option<&Acon> {
		let paths = path.split('.');
		let mut current = self;
		for path in paths {
			let owned = current;
			current = match owned.get(path) {
				Some(ref acon) => acon,
				None => return None,
			}
		}
		Some(current)
	}

	/// Retrieve a mutable reference to an entry via its path.
	/// Paths are dot-separated.
	pub fn path_mut(&mut self, path: &str) -> Option<&mut Acon> {
		let paths = path.split('.');
		let mut current = self;
		for path in paths {
			let owned = current;
			current = match owned.get_mut(path) {
				Some(acon) => acon,
				None => return None,
			}
		}
		Some(current)
	}

	/// Retrieve a reference to an entry
	pub fn get(&self, path: &str) -> Option<&Acon> {
		match *self {
			Acon::Array(ref array) => {
				match path.parse::<usize>() {
					Ok(value) => array.get(value),
					_ => None,
				}
			}
			Acon::String(_) => None,
			Acon::Table(ref table) => table.get(path),
		}
	}

	/// Retrieve a mutable reference to an entry
	pub fn get_mut(&mut self, path: &str) -> Option<&mut Acon> {
		match *self {
			Acon::Array(ref mut array) => {
				match path.parse::<usize>() {
					Ok(value) => array.get_mut(value),
					_ => None,
				}
			}
			Acon::String(_) => None,
			Acon::Table(ref mut table) => table.get_mut(path),
		}
	}
}

/// Errors that come about during parsing
#[derive(PartialEq, Clone, Debug)]
pub enum AconError {
	/// Indicates that there are too many closing delimiters compared to opening
	/// delimiters
	ExcessiveClosingDelimiter(Option<usize>),
	/// Acon::String is the top of the stack. This indicates an interal error
	InternalStringTop(Option<usize>),
	/// The stack top is missing, indicating that something popped the top
	MissingStackTop(Option<usize>),
	/// There is more than one top node after parsing the input. Unterminated tables.
	MultipleTopNodes,
	/// If the top node of the stack is an array, this indicates that there's an
	/// unterminated array
	TopNodeIsArray,
	/// The key at this line is already present in the parent table
	OverwritingKey(Option<usize>),
	/// Got a } but expected a ]
	WrongClosingDelimiterExpectedArray(Option<usize>),
	/// Got a ] but expected a }
	WrongClosingDelimiterExpectedTable(Option<usize>),
}

#[allow(dead_code)]
impl AconError {
	/// Prints a human-friendly error string for the given parse error.
	fn reason(&self) -> String {
		use AconError::*;
		match *self {
			ExcessiveClosingDelimiter(line) => {
				let first = match line { Some(line) => format!("On line {}, t", line), None => "T".to_string() };
				format!("{}here's a closing delimiter that has no matching opening delimiter. Note that
all delimiters must be the first word on a line to count as such. The only delimiters are {}, {}, [, ], and $.",
				first, "{", "}")
			}
			InternalStringTop(line) => {
				let first = match line { Some(line) => format!("On line {}, t", line), None => "T".to_string() };
				format!("{}here's a string on the top of the internal parse stack. This is impossible unless there is a
bug in the parser. Please report this along with the input to the repository maintainer of ACON.", first)
			}
			MissingStackTop(line) => {
				let first = match line { Some(line) => format!("On line {}, t", line), None => "T".to_string() };
				format!("{}he top of the stack is missing. This indicates an internal error, as it's never supposed to
happen. Please contact the maintainer of the ACON repository.", first)
			}
			MultipleTopNodes => {
				"There is an unterminated table, you can append '$' to the input or try terminating it by finding a flaw in the input.".to_string()
			}
			TopNodeIsArray => {
				"The top of the stack is an array. This indicates that there is an unterminated array all the way
until the end of the input. Try appending a ']' to the input to see if this solves the issue.".to_string()
			}
			OverwritingKey(line) => {
				let first = match line { Some(line) => format!("On line {}, t", line), None => "T".to_string() };
				format!("{}he key is already present in the table.", first)
			}
			WrongClosingDelimiterExpectedArray(line) => {
				let first = match line { Some(line) => format!("On line {}, t", line), None => "T".to_string() };
				format!("{}he closing delimiter did not match the array closing delimiter ]. Make sure all delimiters
match up in the input. Some editors can help you by jumping from/to each delimiter.", first)
			}
			WrongClosingDelimiterExpectedTable(line) => {
				let first = match line { Some(line) => format!("On line {}, t", line), None => "T".to_string() };
				format!("{}he closing delimiter did not match the table closing delimiter {}. Make sure all delimiters
until the end of the input. Try appending a ']' to the input to see if this solves the issue.", first, "}")
			}
		}
	}
}

impl std::fmt::Display for Acon {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			Acon::Table(ref table) => {
				for (key, value) in table {
					try!(recurse(key, value, f, 0));
				}
			}
			_ => {
				return Err(std::fmt::Error);
			}
		}
		return Ok(());

		fn recurse(key: &str, acon: &Acon, f: &mut std::fmt::Formatter,
		           depth: usize) -> std::fmt::Result {
			let indent = String::from_utf8(vec![b'\t'; depth]).unwrap();
			macro_rules! wrt {
				( $( $x:expr ),* ) => {{
					try!(f.write_str(&indent));
					$(try!(f.write_str($x));)*
				}
				};
			}
			macro_rules! nl {
				() => { try!(f.write_str("\n")); }
			}
			match *acon {
				Acon::Array(ref array) => {
					wrt!("[ ", key, "\n");
					for value in array {
						try!(recurse("", value, f, depth + 1));
					}
					nl!();
					wrt!("]\n");
				}
				Acon::String(ref string) => {
					wrt!(key, " ", string, "\n");
				}
				Acon::Table(ref table) => {
					wrt!("{ ", key, "\n");
					for (key, value) in table {
						try!(recurse(key, value, f, depth + 1));
					}
					nl!();
					wrt!("}\n");
				}
			}
			Ok(())
		}
	}
}

impl FromStr for Acon {
	type Err = AconError;

	/// Parse a string into an Acon value
	///
	///  ```
	///  use acon::Acon;
	///  let input = r#"
	///    key value
	///    { table-name
	///      key value
	///      key2 value2
	///    }
	///  "#;
	///  let result = input.parse::<Acon>().unwrap();
	///  match result {
	///    Acon::Table(_) => assert!(true),
	///    _ => assert!(false),
	///  }
	///  ```
	///
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut stack = vec![];
		let lines = s.lines();
		let mut current_line = 0usize;
		push_base_table(&mut stack);

		for line in lines {
			current_line += 1;

			let mut words = line.split_whitespace();

			let mut first = None;
			if let Some(word) = words.next() {
				first = Some(word);
				match word {
					"{" => { push_table(&mut words, &mut stack); continue; }
					"[" => { push_array(&mut words, &mut stack); continue; }
					word @ "}" | word @ "]" => { try!(close_array_or_table(word, &mut stack, current_line)); continue; }
					"$" => { try!(close_all_nestings(&mut stack, current_line)); continue; }
					"#" => continue,
					_ => { }
				}
			}

			if let Some(top) = stack.last_mut() {
				match top.value {
					Acon::Array(ref mut array)
						=> { append_line_to_top_array(array, &first, &mut words); }
					Acon::String(_)
						=> return Err(AconError::InternalStringTop(Some(current_line))),
					Acon::Table(ref mut table)
						=> { try!(append_entry_to_top_table(table, &first, &mut words, current_line)); }
				}
			} else {
				return Err(AconError::MissingStackTop(Some(current_line)));
			}
		}

		return {
			if let Some(node) = stack.pop() {
				match node.value {
					Acon::Array(_) => Err(AconError::TopNodeIsArray),
					Acon::String(_) => Err(AconError::InternalStringTop(Some(current_line))),
					Acon::Table(table) => {
						if !stack.is_empty() {
							Err(AconError::MultipleTopNodes)
						} else {
							Ok(Acon::Table(table))
						}
					}
				}
			} else {
				Err(AconError::MissingStackTop(None))
			}
		};


		// BEGIN HELPER STRUCTURE ////////////////////////////////////////////
		use std::str::SplitWhitespace;
		struct Node {
			name: String,
			value: Acon,
		}
		// END HELPER STRUCTURE //////////////////////////////////////////////

		// BEGIN HELPER FUNCTIONS ////////////////////////////////////////////
		fn push_base_table(stack: &mut Vec<Node>) {
			stack.push(Node {
				name: "".to_string(),
				value: Acon::Table(Table::new()),
			});
		}

		fn push_array(words: &mut SplitWhitespace, stack: &mut Vec<Node>) {
			let name = words.next().unwrap_or("");
			stack.push(Node {
				name: name.to_string(),
				value: Acon::Array(Array::new()),
			});
		}

		fn push_table(words: &mut SplitWhitespace, stack: &mut Vec<Node>) {
			let name = words.next().unwrap_or("");
			stack.push(Node {
				name: name.to_string(),
				value: Acon::Table(Table::new()),
			});
		}

		fn close_all_nestings(stack: &mut Vec<Node>, line: usize) -> Result<(), AconError> {
			while stack.len() > 1 {
				if let Some(top) = stack.pop() {
					if let Some(node) = stack.last_mut() {
						match node.value {
							Acon::Array(ref mut array) => {
								if top.name == "" {
									array.push(top.value);
								} else {
									let mut new = Table::new();
									new.insert(top.name, top.value);
									array.push(Acon::Table(new));
								}
							}
							Acon::String(_) => { return Err(AconError::InternalStringTop(Some(line))); }
							Acon::Table(ref mut table) => {
								if table.contains_key(&top.name) {
									return Err(AconError::OverwritingKey(Some(line)));
								}
								table.insert(top.name, top.value);
							}
						}
					}
				}
			}
			Ok(())
		}

		fn close_array_or_table(word: &str, stack: &mut Vec<Node>, line: usize) -> Result<(), AconError> {
			if let Some(top) = stack.pop() {
				match top.value {
					Acon::Array(_) if word != "]"
						=> return Err(AconError::WrongClosingDelimiterExpectedArray(Some(line))),
					Acon::String(_) if word != "]"
						=> return Err(AconError::InternalStringTop(Some(line))),
					Acon::Table(_) if word != "}"
						=> return Err(AconError::WrongClosingDelimiterExpectedTable(Some(line))),
					_ => {}
				}
				if let Some(node) = stack.last_mut() {
					match node.value {
						Acon::Array(ref mut array) => {
							if top.name == "" {
								array.push(top.value);
							} else {
								let mut new = Table::new();
								new.insert(top.name, top.value);
								array.push(Acon::Table(new));
							}
						}
						Acon::String(_) => { return Err(AconError::InternalStringTop(Some(line))); }
						Acon::Table(ref mut table) => {
							if table.contains_key(&top.name) {
								return Err(AconError::OverwritingKey(Some(line)));
							}
							table.insert(top.name, top.value);
						}
					}
					Ok(())
				} else {
					Err(AconError::ExcessiveClosingDelimiter(Some(line)))
				}
			} else {
				Err(AconError::MissingStackTop(Some(line)))
			}
		}

		fn append_line_to_top_array(array: &mut Array,
		                            first: &Option<&str>,
		                            words: &mut SplitWhitespace) {
			let first = first.unwrap_or("");
			let acc = words.fold(first.to_string(), |acc, x| acc + " " + x);
			let acc = acc.trim();
			array.push(Acon::String(acc.to_string()));
		}

		fn append_entry_to_top_table(table: &mut Table,
		                             first: &Option<&str>,
		                             words: &mut SplitWhitespace,
		                             line: usize) -> Result<(), AconError> {
			if let Some(ref key) = *first {
				if table.contains_key(&key.to_string()) {
					return Err(AconError::OverwritingKey(Some(line)));
				}
				let acc = words.fold("".to_string(), |acc, x| acc + " " + x);
				let acc = acc.trim();
				table.insert(key.to_string(), Acon::String(acc.to_string()));
			}
			Ok(())
		}
		// END HELPER FUNCTIONS //////////////////////////////////////////////

	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test::{Bencher, black_box};

	fn key_eq(input: &str, key: &str, string: &str) -> Acon {
		let acon = input.parse::<Acon>().unwrap();
		assert_eq!(acon.path(key).unwrap().string(), string);
		acon
	}

	fn key_eqt(acon: &Acon, key: &str, string: &str) {
		assert_eq!(acon.path(key).unwrap().string(), string);
	}

	#[test]
	fn neg_duplicate_keys() {
		let value = r#"
			key value1
			key2 value2
			key value3
			key2 value4
		"#;
		let acon = value.parse::<Acon>();
		assert_eq!(acon, Err(AconError::OverwritingKey(Some(4))));
	}

	#[test]
	fn neg_duplicate_keys_table() {
		let value = r#"
			key value1
			key2 value2
			{ key
			}
			key2 value4
		"#;
		let acon = value.parse::<Acon>();
		assert_eq!(acon, Err(AconError::OverwritingKey(Some(5))));
	}

	#[test]
	fn neg_duplicate_keys_array() {
		let value = r#"
			key value1
			key2 value2
			[ key
			]
			key2 value4
		"#;
		let acon = value.parse::<Acon>();
		assert_eq!(acon, Err(AconError::OverwritingKey(Some(5))));
	}

	#[test]
	fn neg_duplicate_keys_nested() {
		let value = r#"
			{ key
				{ key
					key value
					[
					]
					key value
				}
			}
		"#;
		let acon = value.parse::<Acon>();
		assert_eq!(acon, Err(AconError::OverwritingKey(Some(7))));
	}

	#[test]
	fn inspect_message() {
		let value = r#"
			[
				{ message
					recipient me
					sender you
					[ content
						Hey what is this ACON thingy all about?
						I mean, we've got TOML, JSON, XML, and SGML.
						Why do we need this data serilization language?
					]
				}
				{ message
					sender me
					recipient you
					[ content
						ACON means Awk-Compatible Object Notation.
						TOML, JSON, etc are great serialization languages, but they're quite complex.
						We need tools and languages that are easily
						parsable and friendly for bash scripting.
						ACON allows just that!
					]
				}
			]
		"#;
		let acon = value.parse::<Acon>().unwrap();
		assert_eq!(acon.get("").unwrap().array().get(1).unwrap().table()
							 .get("message").unwrap().table().get("recipient").unwrap().string(), "you");
		assert_eq!(acon.path(".1.message.recipient"), Some(&Acon::String("you".to_string())));
	}

	#[test]
	fn inspect_dollar_closing() {
		let value = r#"
		{ table
			{ table
				{ table
					[ array
						{ table
							key value

		$ This word as the first word on a line closes all nestings

		[ reason
			I want to get rid of it all.
			If a program crashes whilst serializing (like a script that
			gets an error). Then another program can append $ to the
			end of the stream, clearing that stream.
		]
		"#;
		key_eq(value, "table.table.table.array.0.table.key", "value");
	}

	#[test]
	fn dollar_closing_array_whitespace() {
		let value = r#"
		[ array



		$
		"#;
		let acon = value.parse::<Acon>().unwrap();
		assert_eq!(acon.path("array.2"), Some(&Acon::String("".to_string())));
	}

	#[test]
	fn dollar_duplicate() {
		let value = r#"
		{ table
			key value

		$
		{ table

		$
		"#;
		let acon = value.parse::<Acon>();
		assert_eq!(acon, Err(AconError::OverwritingKey(Some(8))));
	}

	#[test]
	fn neg_ending_array() {
		let value = r#"
		[ array
			value

		"#;
		let acon = value.parse::<Acon>();
		assert_eq!(acon, Err(AconError::TopNodeIsArray));
	}

	#[test]
	fn neg_ending_table() {
		let value = r#"
		{ table
			key value

		"#;
		let acon = value.parse::<Acon>();
		assert_eq!(acon, Err(AconError::MultipleTopNodes));
	}

	#[test]
	fn unnamed_table() {
		let value = r#"
		{
			key value
		}
		"#;
		key_eq(value, ".key", "value");
	}

	#[test]
	fn unnamed_table_2() {
		let value = r#"
		{ named
			key value
		}
		"#;
		key_eq(value, "named.key", "value");
	}

	#[test]
	fn unnamed_array() {
		let value = r#"
		[
			[
				[
					0
		$
		"#;
		key_eq(value, ".0.0.0", "0");
	}

	#[test]
	fn unnamed_array_2() {
		let value = r#"
		[
			[
				[ name
					0
		$
		"#;
		key_eq(value, ".0.0.name.0", "0");
	}

	#[test]
	fn unnamed_elements() {
		let value = r#"
			{ a
				{
					b c
				}
			}
		"#;
		key_eq(value, "a..b", "c");
	}

	#[test]
	fn similarity_acon() {
		let value = r#"
			{ menu
				id file
				value File
				{ popup
					[ menuitem
						{
							value New
							onclick CreateNewDoc()
						}
						{
							value Open
							onclick OpenDoc()
						}
						{
							value Close
							onclick CloseDoc()
						}
					]
				}
			}
		"#;
		key_eq(value, "menu.popup.menuitem.2.value", "Close");
	}

	#[test]
	fn dot_separation() {
		let value = r#"
			{
				{
					lorem ipsum
				}
			}
		"#;
		key_eq(value, "..lorem", "ipsum");
	}

	#[test]
	fn dot_separation_in_array() {
		let value = r#"
			[
				{
					lorem ipsum
				}
			]
		"#;
		key_eq(value, ".0.lorem", "ipsum");
	}

	#[test]
	fn dot_separation_in_array_named_table() {
		let value = r#"
			[
				{ dolor
					lorem ipsum
				}
			]
		"#;
		key_eq(value, ".0.dolor.lorem", "ipsum");
	}

	#[test]
	fn attempt_edges() {
		let value = r#"
			lorem ipsum
			{ dolor
				sit amet
			}
			[ deleniti
			placeat quia
			]
			[
				[
					{ ipsam
						beatae vel
						Iusto enim
					}
				]
				[
					{
						aut quidem
						Sit vitae
					}
				]
			]
		"#;
		let acon = key_eq(value, ".1.0.Sit", "vitae");
		key_eqt(&acon, "deleniti.0", "placeat quia");
	}

	#[test]
	fn named_array_in_unnamed_array() {
		let value = r#"
			[
				[ lorem
					ipsum
				]
			]
		"#;
		key_eq(value, ".0.lorem.0", "ipsum");
	}

	#[test]
	fn named_table_in_unnamed_array() {
		let value = r#"
			[
				{ lorem
					ipsum dolor
				}
			]
		"#;
		key_eq(value, ".0.lorem.ipsum", "dolor");
	}

	#[test]
	fn comment() {
		let value = r#"
			# Comment
			[
				{ lorem
					ipsum dolor
				}
			]
		"#;
		let parsed = key_eq(value, ".0.lorem.ipsum", "dolor");
		assert_eq!(parsed.table().contains_key("#"), false);
	}

	#[test]
	fn table_comment() {
		let value = r#"
			[
				{ lorem
					# sit amet
					ipsum dolor
					# echo alpha
			$ # sequi
		"#;
		let parsed = key_eq(value, ".0.lorem.ipsum", "dolor");
		assert_eq!(parsed.table().contains_key("#"), false);
		assert_eq!(parsed.table().contains_key("$"), false);
	}

	#[bench]
	fn large_table(bench: &mut Bencher) {
		use std::fs::File;
		use std::io::Read;

		let mut stream = match File::open("lorem ipsum") {
			Ok(stream) => stream,
			Err(_) => { println!("{}", "Unable to open file, skipping"); return; }
		};
		let mut string = String::new();
		stream.read_to_string(&mut string).expect("Unable to read file to memory, run contamine to create the file");

		fn parse(string: &str) -> Acon {
			string.parse::<Acon>().unwrap()
		}

		bench.iter(|| {
			black_box(parse(&string));
		});
	}

	#[bench]
	fn split_complexity(bench: &mut Bencher) {
		use std::fs::File;
		use std::io::Read;

		let mut stream = match File::open("lorem ipsum") {
			Ok(stream) => stream,
			Err(_) => { println!("{}", "Unable to open file, skipping"); return; }
		};
		let mut string = String::new();
		stream.read_to_string(&mut string).expect("Unable to read file to memory, run contamine to create the file");

		fn split(string: &str) {
			let _ = string.split(' ');
		}

		bench.iter(|| {
			black_box(split(&string));
		});
	}
}
