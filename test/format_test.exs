defmodule MDEx.FormatTest do
  use ExUnit.Case
  doctest MDEx

  @md_opts extension: [front_matter_delimiter: "---", table: true, tasklist: true, autolink: true, shortcodes: true]

  def assert_format(document, expected) do
    ast = MDEx.parse_document(document, @md_opts)
    assert MDEx.to_html(ast, @md_opts) == expected
  end

  test "text" do
    assert_format("mdex", "<p>mdex</p>\n")
  end

  test "headings" do
    assert_format(
      """
      # one
      ## two
      ### three
      """,
      "<h1>one</h1>\n<h2>two</h2>\n<h3>three</h3>\n"
    )
  end

  test "code block" do
    assert_format(
      """
      ```elixir
      String.trim(" MDEx ")
      ```
      """,
      "<pre><code class=\"language-elixir\">String.trim(&quot; MDEx &quot;)\n</code></pre>\n"
    )
  end

  test "table" do
    assert_format(
      """
      | foo | bar |
      | --- | --- |
      | baz | bim |
      """,
      "<table>\n<thead>\n<tr>\n<th>foo</th>\n<th>bar</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>baz</td>\n<td>bim</td>\n</tr>\n</tbody>\n</table>\n"
    )
  end
end
