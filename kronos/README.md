# kronos
A library (WIP) to work with time expressions.

Try out the **kronos** example binary that uses both *kronos* and *earlgrey* to parse some text into time objects.
Run *cargo test* to build the example.

```
$ kronos next monday 28th of feb

* "<time> -> next <seq>" ==> "next | <seq> -> <base_seq> <seq>"
* "<seq> -> <base_seq> <seq>" ==> "<base_seq> -> <day-of-week> | <seq> -> <base_seq> of <seq>"
* "<base_seq> -> <day-of-week>" ==> "monday"
* "<seq> -> <base_seq> of <seq>" ==> "<base_seq> -> <day-of-month> | of | <seq> -> <base_seq>"
* "<base_seq> -> <day-of-month>" ==> "28th"
* "<seq> -> <base_seq>" ==> "<base_seq> -> <named-month>"
* "<base_seq> -> <named-month>" ==> "feb"

Range { start: 2022-02-28T00:00:00, end: 2022-03-01T00:00:00, grain: Day }
```

### references
* https://duckling.wit.ai/
* http://homes.cs.washington.edu/~kentonl/pub/ladz-acl.2014.pdf
