# kronos
Compute *time expressions*.

Example:
- the 2nd monday of April: `Seq::nthof(2, Seq::weekday(1), Seq::month(4))`
- 3rd week of June: `Seq::nthof(3, Seq::from_grain(Grain::Week), Seq::month(6))`
- last weekend of the year: `Seq::lastof(1, Seq::weekend(), Seq::from_grain(Grain::Year))`
- a sequence of all days *monday 28th*:
```
Seq::intersect(Seq::weekday(1),
               Seq::nthof(28, Seq::from_grain(Grain::Day),
               Seq::from_grain(Grain::Month)))
```


### references
* https://duckling.wit.ai/
* http://homes.cs.washington.edu/~kentonl/pub/ladz-acl.2014.pdf
