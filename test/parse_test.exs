defmodule MDEx.ParseTest do
  use ExUnit.Case
  doctest MDEx

  @md_opts extension: [front_matter_delimiter: "---", table: true, tasklist: true, autolink: true, shortcodes: true]

  def assert_parse_document(document, expected) do
    assert MDEx.parse_document(document, @md_opts) == expected
  end

  test "text" do
    assert_parse_document("mdex", [{"document", [], [{"paragraph", [], ["mdex"]}]}])
  end

  test "code block" do
    assert_parse_document(
      """
      ```elixir
      String.trim(" MDEx ")
      ```
      """,
      [
        {"document", [],
         [
           {"code_block",
            [
              {"fenced", true},
              {"fence_char", "`"},
              {"fence_length", 3},
              {"fence_offset", 0},
              {"info", "elixir"},
              {"literal", "String.trim(\" MDEx \")\n"}
            ], []}
         ]}
      ]
    )
  end

  test "table" do
    assert_parse_document(
      """
      | foo | bar |
      | --- | --- |
      | baz | bim |
      """,
      [
        {"document", [],
         [
           {"table", [{"alignments", ["none", "none"]}, {"num_columns", 2}, {"num_rows", 1}, {"num_nonempty_cells", 2}],
            [
              {"table_row", [{"header", true}], [{"table_cell", [], ["foo"]}, {"table_cell", [], ["bar"]}]},
              {"table_row", [{"header", false}], [{"table_cell", [], ["baz"]}, {"table_cell", [], ["bim"]}]}
            ]}
         ]}
      ]
    )

    assert_parse_document(
      """
      | abc | defghi |
      :-: | -----------:
      bar | baz
      """,
      [
        {"document", [],
         [
           {"table", [{"alignments", ["center", "right"]}, {"num_columns", 2}, {"num_rows", 1}, {"num_nonempty_cells", 2}],
            [
              {"table_row", [{"header", true}], [{"table_cell", [], ["abc"]}, {"table_cell", [], ["defghi"]}]},
              {"table_row", [{"header", false}], [{"table_cell", [], ["bar"]}, {"table_cell", [], ["baz"]}]}
            ]}
         ]}
      ]
    )
  end
end
