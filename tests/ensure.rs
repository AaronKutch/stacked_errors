use stacked_errors::{ensure_eq, ensure_ne, Result, UnitError};

// there are also tests in the doc tests
#[test]
#[allow(clippy::partialeq_ne_impl)]
fn ensure_test() {
    // this is to test that movement within the macros do not result in errors, and
    // other properties
    #[derive(Debug)]
    struct X(i8);
    impl X {
        fn mov(self) -> X {
            self
        }
    }
    impl PartialEq for X {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }

        fn ne(&self, _: &Self) -> bool {
            panic!()
        }
    }
    #[derive(Debug)]
    struct Y(i8);
    impl Y {
        fn mov(self) -> Y {
            self
        }
    }
    impl PartialEq for Y {
        fn eq(&self, _: &Self) -> bool {
            panic!()
        }

        fn ne(&self, other: &Self) -> bool {
            self.0 != other.0
        }
    }

    fn successes() -> Result<u8> {
        let lhs = X(2);
        let rhs = X(2);
        ensure_eq!(lhs.mov(), rhs.mov());
        let lhs = X(2);
        let rhs = X(2);
        ensure_eq!(lhs.mov(), rhs.mov(), "hello");
        let lhs = Y(1);
        let rhs = Y(2);
        ensure_ne!(lhs.mov(), rhs.mov());
        let lhs = Y(1);
        let rhs = Y(2);
        ensure_ne!(lhs.mov(), rhs.mov(), "hello");
        ensure_eq!(String::new(), "");
        ensure_ne!("1".to_owned(), "2");
        let () = ensure_eq!(String::new(), "", "hello");
        ensure_ne!("1".to_owned(), "2", UnitError {});
        Ok(0)
    }
    assert_eq!(successes().unwrap(), 0);

    let fail = || -> Result<u8> {
        ensure_eq!(1, 2);
        Ok(0)
    };
    println!("{:?}", fail().unwrap_err());
    assert_eq!(
        format!("{}", fail().unwrap_err()),
        convert(
            r#"
    ensure_eq(
 lhs: 1
 rhs: 2
) -> equality assertion failed at tests/ensure.rs 64:9"#
        )
    );

    let fail = || -> Result<u8> {
        ensure_eq!(1, 2, "hello");
        Ok(0)
    };
    println!("{:?}", fail().unwrap_err());
    assert_eq!(
        format!("{}", fail().unwrap_err()),
        convert(
            r#"
    hello at tests/ensure.rs 80:9"#
        )
    );

    let fail = || -> Result<u8> {
        ensure_ne!(2, 2);
        Ok(0)
    };
    println!("{:?}", fail().unwrap_err());
    assert_eq!(
        format!("{}", fail().unwrap_err()),
        convert(
            r#"
    ensure_ne(
 lhs: 2
 rhs: 2
) -> inequality assertion failed
  at tests/ensure.rs 93:9"#
        )
    );

    let fail = || -> Result<u8> {
        ensure_ne!(2, 2, "hello");
        Ok(0)
    };
    println!("{:?}", fail().unwrap_err());
    assert_eq!(
        format!("{}", fail().unwrap_err()),
        convert(
            r#"
    hello at tests/ensure.rs 110:9"#
        )
    );
}

pub fn convert(s: &str) -> String {
    let s = s.to_owned();
    if cfg!(windows) {
        s.replace("/", "\\")
    } else {
        s
    }
}
