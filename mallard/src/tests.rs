
#[cfg(test)]
mod tests {
    use chrono::naive::date::NaiveDate as Date;
    #[test]
    fn test_time_1() {
        let s = "july 2015";
        let s = "june 2014";
        let s = "last feb of 2013";
        let s = "last day of feb 2013";
        let s = "july 23rd";
        let s = "last week of feb";
        let s = "2nd month of 2012";
        let s = "3rd day of june";
        let s = "last day of feb next year";
        let s = "mondays of june";

        let s = "mon feb 28th"; // slow
        let s = "2nd thu of sep 2016";
        let s = "3 days after mon feb 28th";

        let s = "1st thu of the month"; // OK, it's != this month
        let s = "1st thu of this month";

        let s = "feb next year"; // doesn't work
        let s = "4th day of next year"; // doesn't work
        let s = "the 2nd day, of the 3rd week, of february"; // some branches don't finish
    }
}
