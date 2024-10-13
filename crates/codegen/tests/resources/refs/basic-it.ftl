-term = it
hello = ciao
hello-name = { -term }: { hello } { $name }

-term-param = hello { $foo }
term-reference-with-named-param = { $bar } { -term-param(foo: "it")}
two-term-refs = { -term } { -term-param(foo:"it")}
-nested-term = nested { -term-param(foo: "bar") }
nested-term-ref = { -term } { -nested-term }

-with-attrs = { -term }
  .simple-attr = simple
  .param-attr = attr { $param }

term-with-attrs = { -term } { -with-attrs }

message-as-arg-value = { -term } { -term-param(foo:hello)}
