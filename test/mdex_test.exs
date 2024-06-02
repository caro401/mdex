defmodule MDExTest do
  use ExUnit.Case
  doctest MDEx

  defp assert_output(input, expected, opts \\ []) do
    html = MDEx.to_html(input, opts)
    assert html == expected
  end

  @long_md """
  ---
  title :test
  ---

  It works! :smile:

  # heading 1
  ## heading 2

  https://github.com/leandrocp/mdex
  [MDEx](https://github.com/leandrocp/mdex "mdex")

  ![logo](https://raw.githubusercontent.com/leandrocp/mdex/main/assets/images/mdex_logo.png "mdex")

  > block quote

  _emph_

  **strong**
    - Elixir
    - Rust

  ```elixir
  String.trim(" MDEx ")
  ```

  code: `Atom.to_string(:elixir)`

  <div>
    <span>html</span>
  <div>

  * [x] Done
  * [ ] Not done
  """

  @md_opts extension: [front_matter_delimiter: "---", table: true, tasklist: true, autolink: true, shortcodes: true]

  def assert_parse_document(document, expected) do
    assert MDEx.parse_document(document, @md_opts) == expected
  end

  def assert_format(document, expected) do
    ast = MDEx.parse_document(document, @md_opts)
    assert MDEx.to_html(ast, @md_opts) == expected
  end

  describe "parse" do
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

  describe "tree to_html" do
    test "wrap fragment in root document" do
      assert MDEx.to_html([{"paragraph", [], ["mdex"]}]) == "<p>mdex</p>\n"
      assert MDEx.to_html(["mdex", "test"]) == "<p>mdextest</p>\n"
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

  test "parse" do
    assert MDEx.parse_document(@long_md, extension: [front_matter_delimiter: "---", table: true, tasklist: true, autolink: true, shortcodes: true]) ==
             [
               {"document", [],
                [
                  {"front_matter", [{"content", "---\ntitle :test\n---\n\n"}], []},
                  {"paragraph", [], ["It works! ", {"short_code", [{"name", "smile"}, {"emoji", "😄"}], []}]},
                  {"heading", [{"level", 1}, {"setext", false}], ["heading 1"]},
                  {"heading", [{"level", 2}, {"setext", false}], ["heading 2"]},
                  {"paragraph", [],
                   [
                     "",
                     {"link", [{"url", "https://github.com/leandrocp/mdex"}, {"title", ""}], ["https://github.com/leandrocp/mdex"]},
                     {"soft_break", [], []},
                     {"link", [{"url", "https://github.com/leandrocp/mdex"}, {"title", "mdex"}], ["MDEx"]}
                   ]},
                  {"paragraph", [],
                   [
                     {"image", [{"url", "https://raw.githubusercontent.com/leandrocp/mdex/main/assets/images/mdex_logo.png"}, {"title", "mdex"}],
                      ["logo"]}
                   ]},
                  {"block_quote", [], [{"paragraph", [], ["block quote"]}]},
                  {"paragraph", [], [{"emph", [], ["emph"]}]},
                  {"paragraph", [], [{"strong", [], ["strong"]}]},
                  {"list",
                   [
                     {"list_type", "bullet"},
                     {"marker_offset", 2},
                     {"padding", 2},
                     {"start", 1},
                     {"delimiter", "period"},
                     {"bullet_char", "-"},
                     {"tight", true}
                   ],
                   [
                     {"item",
                      [
                        {"list_type", "bullet"},
                        {"marker_offset", 2},
                        {"padding", 2},
                        {"start", 1},
                        {"delimiter", "period"},
                        {"bullet_char", "-"},
                        {"tight", false}
                      ], [{"paragraph", [], ["Elixir"]}]},
                     {"item",
                      [
                        {"list_type", "bullet"},
                        {"marker_offset", 2},
                        {"padding", 2},
                        {"start", 1},
                        {"delimiter", "period"},
                        {"bullet_char", "-"},
                        {"tight", false}
                      ], [{"paragraph", [], ["Rust"]}]}
                   ]},
                  {"code_block",
                   [
                     {"fenced", true},
                     {"fence_char", "`"},
                     {"fence_length", 3},
                     {"fence_offset", 0},
                     {"info", "elixir"},
                     {"literal", "String.trim(\" MDEx \")\n"}
                   ], []},
                  {"paragraph", [], ["code: ", {"code", [{"num_backticks", 1}, {"literal", "Atom.to_string(:elixir)"}], []}]},
                  {"html_block", [{"block_type", 6}, {"literal", "<div>\n  <span>html</span>\n<div>\n"}], []},
                  {"list",
                   [
                     {"list_type", "bullet"},
                     {"marker_offset", 0},
                     {"padding", 2},
                     {"start", 1},
                     {"delimiter", "period"},
                     {"bullet_char", "*"},
                     {"tight", true}
                   ],
                   [
                     {"task_item", [{"symbol", "x"}], [{"paragraph", [], ["Done"]}]},
                     {"task_item", [{"symbol", " "}], [{"paragraph", [], ["Not done"]}]}
                   ]}
                ]}
             ]
  end

  test "format" do
    ast = MDEx.parse_document(@long_md, extension: [front_matter_delimiter: "---", table: true, tasklist: true, autolink: true, shortcodes: true])

    assert MDEx.to_html(ast) == """
           <p>It works! 🚀</p>
           <h1>heading 1</h1>
           <h2>heading 2</h2>
           <p><a href="TODO" title="TODO">https://github.com/leandrocp/mdex</a>
           <a href="TODO" title="TODO">MDEx</a></p>
           <p><img src="TODO" alt="logo" title="TODO" /></p>
           <blockquote>
           <p>block quote</p>
           </blockquote>
           <p><em>emph</em></p>
           <p><strong>strong</strong></p>
           <ul>
           <li>Elixir</li>
           <li>Rust</li>
           </ul>
           <pre><code class="language-elixir">String.trim(&quot; MDEx &quot;)\n</code></pre>
           <p>code: <code>TODO</code></p>
           <!-- raw HTML omitted -->
           <ul>
           <li><input type="checkbox" checked="" disabled="" /> Done</li>
           <li><input type="checkbox" checked="" disabled="" /> Not done</li>
           </ul>
           """
  end

  describe "syntax highlighting" do
    test "enabled by default" do
      assert_output(
        ~S"""
        ```elixir
        {:mdex, "~> 0.1"}
        ```
        """,
        ~S"""
        <pre class="autumn-hl" style="background-color: #282C34; color: #ABB2BF;"><code class="language-elixir" translate="no"><span class="ahl-punctuation ahl-bracket" style="color: #ABB2BF;">{</span><span class="ahl-string ahl-special ahl-symbol" style="color: #98C379;">:mdex</span><span class="ahl-punctuation ahl-delimiter" style="color: #ABB2BF;">,</span> <span class="ahl-string" style="color: #98C379;">&quot;~&gt; 0.1&quot;</span><span class="ahl-punctuation ahl-bracket" style="color: #ABB2BF;">}</span>
        </code></pre>
        """
      )
    end

    test "change theme name" do
      assert_output(
        ~S"""
        ```elixir
        {:mdex, "~> 0.1"}
        ```
        """,
        ~S"""
        <pre class="autumn-hl" style="background-color: #2e3440; color: #D8DEE9;"><code class="language-elixir" translate="no"><span class="ahl-punctuation ahl-bracket" style="color: #ECEFF4;">{</span><span class="ahl-string ahl-special ahl-symbol" style="color: #EBCB8B;">:mdex</span><span class="ahl-punctuation ahl-delimiter" style="color: #ECEFF4;">,</span> <span class="ahl-string" style="color: #A3BE8C;">&quot;~&gt; 0.1&quot;</span><span class="ahl-punctuation ahl-bracket" style="color: #ECEFF4;">}</span>
        </code></pre>
        """,
        features: [syntax_highlight_theme: "nord"]
      )
    end

    test "can be disabled" do
      assert_output(
        ~S"""
        ```elixir
        {:mdex, "~> 0.1"}
        ```
        """,
        ~S"""
        <pre><code class="language-elixir">{:mdex, &quot;~&gt; 0.1&quot;}
        </code></pre>
        """,
        features: [syntax_highlight_theme: nil]
      )
    end

    test "with invalid lang" do
      assert_output(
        ~S"""
        ```invalid
        {:mdex, "~> 0.1"}
        ```
        """,
        ~s"""
        <pre class="autumn-hl" style="background-color: #282C34; color: #ABB2BF;"><code class="language-plaintext" translate="no">{:mdex, &quot;~&gt; 0.1&quot;}
        </code></pre>
        """
      )
    end

    test "without lang" do
      assert_output(
        ~S"""
        ```
        {:mdex, "~> 0.1"}
        ```
        """,
        ~s"""
        <pre class="autumn-hl" style="background-color: #282C34; color: #ABB2BF;"><code class="language-plaintext" translate="no">{:mdex, &quot;~&gt; 0.1&quot;}
        </code></pre>
        """
      )
    end
  end
end
