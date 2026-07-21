simple-number = it { NUMBER($mynum)}
number-const = it { NUMBER(424242) }
number-with-named-arg = it { NUMBER($mynum, useGrouping:"auto")}

message-test = 200
number-msg-ref = it {NUMBER(message-test)}

-term-test = 111
number-term-ref = it term { NUMBER(-term-test) }

selector-number = { NUMBER($mynum) ->
  [0] it zero
  *[other] it other
}

number-number = it inception { NUMBER(NUMBER($mynum)) }

-term-arg = it { $arg }
term-arg-msg = it msg { -term-arg(arg:NUMBER($mynum)) }
