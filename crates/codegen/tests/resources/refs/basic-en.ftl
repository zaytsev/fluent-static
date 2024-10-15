-term = en
hello = hello
hello-name = { -term }: { hello } { $name }

-term-param = hello { $foo }
term-reference-with-named-param = { $bar } { -term-param(foo: "foobar")}
two-term-refs = { -term } { -term-param(foo:1)}
-nested-term = nested { -term-param(foo: "bar") }
nested-term-ref = { -term } { -nested-term }

-with-attrs = { -term }
  .simple-attr = simple
  .param-attr = attr { $param }

term-with-attrs = { -term } { -with-attrs }

message-as-arg-value = { -term } { -term-param(foo:hello)}
