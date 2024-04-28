defmodule MDExTest do
  use ExUnit.Case
  doctest MDEx

  defp assert_output(input, expected, opts \\ []) do
    html = MDEx.to_html(input, opts)
    assert html == expected
  end

  test "parse" do
    assert MDEx.parse_document(
             """
             ---
             title :test
             ---

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

             | foo | bar |
             | --- | --- |
             | baz | bim |

             * [x] Done
             * [ ] Not done

             """,
             extension: [front_matter_delimiter: "---", table: true, tasklist: true, autolink: true]
           ) ==
             {"document", [],
              [
                {"front_matter", [{"content", "---\ntitle :test\n---\n\n"}], []},
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
                {"table", [{"alignments", ["center"]}, {"num_columns", 2}, {"num_rows", 1}, {"num_nomempty_cells", 2}],
                 [
                   {"table_row", [{"header", "true"}], [{"table_cell", [], ["foo"]}, {"table_cell", [], ["bar"]}]},
                   {"table_row", [{"header", "false"}], [{"table_cell", [], ["baz"]}, {"table_cell", [], ["bim"]}]}
                 ]},
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
  end

  test "format" do
    ast =
      MDEx.parse_document("""
        # hello
        ## world

        _It works_

        **Languages**
        - Elixir
        - Rust
      """)

    assert MDEx.to_html(ast) == """
           <h1>hello</h1>
           <h1>world</h1>
           <p><em>It works</em></p>
           <p><strong>Languages</strong></p>
           <ul>
           <ul>
           Elixir</ul>
           <ul>
           Rust</ul>
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
