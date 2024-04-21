defmodule MDExTest do
  use ExUnit.Case
  doctest MDEx

  defp assert_output(input, expected, opts \\ []) do
    html = MDEx.to_html(input, opts)
    assert html == expected
  end

  test "parse" do
    assert MDEx.parse_document("""
             # hello
             ## world

             _It works_

             **Languages**
             - Elixir
             - Rust
           """) == {
             "document",
             [],
             [
               {"heading", [{"level", 1}, {"setext", false}], ["hello"]},
               {"heading", [{"level", 2}, {"setext", false}], ["world"]},
               {"paragraph", [], [{"emph", [], ["It works"]}]},
               {"paragraph", [], [{"strong", [], ["Languages"]}]},
               {"list",
                [
                  {"list_type", "bullet"},
                  {"marker_offset", 2},
                  {"padding", 2},
                  {"start", 1},
                  {"delimiter", "period"},
                  {"bullet_char", 45},
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
                     {"bullet_char", 45},
                     {"tight", false}
                   ], [{"paragraph", [], ["Elixir"]}]},
                  {"item",
                   [
                     {"list_type", "bullet"},
                     {"marker_offset", 2},
                     {"padding", 2},
                     {"start", 1},
                     {"delimiter", "period"},
                     {"bullet_char", 45},
                     {"tight", false}
                   ], [{"paragraph", [], ["Rust"]}]}
                ]}
             ]
           }
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
