use time::{macros::time, Date, OffsetDateTime, PrimitiveDateTime, Time};

pub fn get_week_start_end(base_date: Option<OffsetDateTime>) -> (OffsetDateTime, OffsetDateTime) {
    let base_date = match base_date {
        Some(date) => date,
        None => OffsetDateTime::now_utc(),
    };

    let week_start = PrimitiveDateTime::new(
        Date::from_iso_week_date(
            base_date.year(),
            base_date.iso_week(),
            time::Weekday::Monday,
        )
        .unwrap(),
        Time::MIDNIGHT,
    )
    .assume_utc();

    let week_end = PrimitiveDateTime::new(
        Date::from_iso_week_date(
            base_date.year(),
            base_date.iso_week(),
            time::Weekday::Sunday,
        )
        .unwrap(),
        time!(23:59:59),
    )
    .assume_utc();

    (week_start, week_end)
}

pub fn format_date(date: OffsetDateTime) -> String {
    format!("{}-{}-{}", date.month(), date.day(), date.year())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use time::Month;

    #[derive(Debug)]
    struct DateTuple(i32, Month, u8);

    impl DateTuple {
        fn to_datetime(&self) -> OffsetDateTime {
            Date::from_calendar_date(self.0, self.1, self.2)
                .unwrap()
                .midnight()
                .assume_utc()
        }
    }

    #[rstest]
    #[case(
        DateTuple(2022, Month::August, 6),
        DateTuple(2022, Month::August, 1),
        DateTuple(2022, Month::August, 7)
    )]
    #[case(
        DateTuple(2022, Month::August, 1),
        DateTuple(2022, Month::August, 1),
        DateTuple(2022, Month::August, 7)
    )]
    #[case(
        DateTuple(2022, Month::August, 7),
        DateTuple(2022, Month::August, 1),
        DateTuple(2022, Month::August, 7)
    )]
    fn verify_week_start_end(
        #[case] date: DateTuple,
        #[case] start_date: DateTuple,
        #[case] end_date: DateTuple,
    ) {
        let start_date = start_date.to_datetime();
        let end_date = end_date.to_datetime().replace_time(time!(23:59:59));

        assert_eq!(
            get_week_start_end(Some(date.to_datetime())),
            (start_date, end_date)
        );

        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        let hour = rng.gen_range(0..24);

        assert_eq!(
            get_week_start_end(Some(date.to_datetime().replace_hour(hour).unwrap())),
            (start_date, end_date)
        );
    }

    #[rstest]
    #[case(DateTuple(2022, Month::August, 6), "August-6-2022")]
    fn verify_format_date_works(#[case] date: DateTuple, #[case] expected_output: &str) {
        let output = format_date(date.to_datetime());
        assert_eq!(output, expected_output);
    }
}
