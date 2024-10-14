simple-number = en { NUMBER($mynum) }
number-const = en { NUMBER(4242) }
number-with-named-arg = en { NUMBER($mynum, useGrouping:"auto") }

message-test = 100
number-msg-ref = en { NUMBER(message-test) }

-term-test = 111
number-term-ref = en term { NUMBER(-term-test) }

selector-number = { NUMBER($mynum) ->
  [0] en zero
  *[other] en other
}

number-number = en inception { NUMBER(NUMBER($mynum)) }

-term-arg = en { $arg }
term-arg-msg = en msg { -term-arg(arg:NUMBER($mynum)) }
