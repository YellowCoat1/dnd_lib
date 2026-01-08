use super::choice::{chosen, chosen_ref, split, PresentedOption};

#[test]
fn test_chosen() {
    let presented = vec![
        PresentedOption::Base("Option 1"),
        PresentedOption::Base("Option 2"),
        PresentedOption::Choice(vec!["Option 3"]),
        PresentedOption::Base("Option 4"),
        PresentedOption::Choice(vec!["Option 5", "Option 6"]),
    ];
    let chosen_options = chosen(&presented);
    assert_eq!(chosen_options, vec![&"Option 1", &"Option 2", &"Option 4"]);
}

#[test]
fn test_chosen_ref() {
    fn str_ref(s: &PresentedOption<String>) -> &String {
        if let PresentedOption::Base(str_val) = s {
            str_val
        } else {
            panic!("Expected a Base option");
        }
    }

    let option1 = PresentedOption::Base("Option A".to_string());
    let option1_str = str_ref(&option1);
    let option2 = PresentedOption::Choice(vec!["Option B".to_string()]);
    let option3 = PresentedOption::Base("Option C".to_string());
    let option3_str = str_ref(&option3);
    let presented = vec![&option1, &option2, &option3];
    let chosen_options = chosen_ref(&presented);
    assert_eq!(chosen_options, vec![option1_str, option3_str]);
}

#[test]
fn test_split() {
    let presented = vec![
        PresentedOption::Base("Option 1"),
        PresentedOption::Base("Option 2"),
        PresentedOption::Choice(vec!["Option 3"]),
        PresentedOption::Base("Option 4"),
        PresentedOption::Choice(vec!["Option 5", "Option 6"]),
    ];
    let (chosen_options, choice_options) = split(&presented);
    assert_eq!(chosen_options, vec![&"Option 1", &"Option 2", &"Option 4"]);
    assert_eq!(
        choice_options,
        vec![&vec!["Option 3"], &vec!["Option 5", "Option 6"]]
    );
}

#[test]
fn choose() {
    let option1 = PresentedOption::Choice(vec!["Option 1", "Option 2"]);
    let option1_result = option1.choose(0).expect("Expected a valid choice");
    assert_eq!(*option1_result, "Option 1");

    let option2 = PresentedOption::Choice(vec!["Option A", "Option B", "Option C"]);
    let result = option2.choose(3);
    assert!(result.is_none());

    let mut option3 = PresentedOption::Choice(vec!["Option X", "Option Y"]);
    let result = option3.choose_in_place(0);
    assert!(result, "expected choose_in_place to succeed");
    assert_eq!(option3, PresentedOption::Base("Option X"));

    let mut option4 = PresentedOption::Choice(vec!["Option M", "Option N"]);
    let result = option4.choose_in_place(5);
    assert!(!result, "expected choose_in_place to fail");

    let mut option5 = PresentedOption::Base("Option Z");
    let result = option5.choose_in_place(0);
    assert!(!result, "expected choose_in_place to fail on Base option");
}

#[test]
fn as_base_tests() {
    let option1 = PresentedOption::Base("Base Option");
    assert_eq!(option1.as_base(), Some(&"Base Option"));
    assert_eq!(option1.choices(), None);
    let option2 = PresentedOption::Choice(vec!["Choice 1", "Choice 2"]);
    assert_eq!(option2.as_base(), None);
    assert_eq!(
        option2.choices(),
        Some(vec!["Choice 1", "Choice 2"].as_slice())
    );

    let mut option3 = PresentedOption::Base(1);
    assert_eq!(option3.choices_mut(), None);
    let base_mut = option3
        .as_base_mut()
        .expect("Expected as_base_mut to return Some");
    *base_mut = 5;
    assert_eq!(option3.as_base(), Some(&5));

    let mut option4 = PresentedOption::Choice(vec![10, 20, 30]);
    assert_eq!(option4.as_base_mut(), None);
    let choices_mut = option4
        .choices_mut()
        .expect("Expected choices_mut to return Some");
    choices_mut[0] = 15;
    assert_eq!(option4.choices(), Some(vec![15, 20, 30].as_slice()));
}

#[test]
fn map_base() {
    let option = PresentedOption::Base(10);
    let mapped_option = option.map(|x| x * 2).map(|x| x.to_string());
    assert_eq!(mapped_option, PresentedOption::Base("20".to_string()));
}

#[test]
fn collect() {
    let option = PresentedOption::Base(Some(15));
    let collected_option = option.collect_option();
    assert_eq!(collected_option, Some(PresentedOption::Base(15)));
    let option: PresentedOption<Option<()>> = PresentedOption::Base(None);
    let collected_option = option.collect_option();
    assert_eq!(collected_option, None);

    let option = PresentedOption::Choice(vec![Some(1), Some(2)]);
    let collected_option = option.collect_option();
    assert_eq!(collected_option, Some(PresentedOption::Choice(vec![1, 2])));
    let option = PresentedOption::Choice(vec![Some(1), None]);
    let collected_option = option.collect_option();
    assert_eq!(collected_option, None);

    let option: PresentedOption<Result<i32, ()>> = PresentedOption::Base(Ok(15));
    let collected_option = option.collect_result();
    assert_eq!(collected_option, Ok(PresentedOption::Base(15)));
    let option: PresentedOption<Result<(), &str>> = PresentedOption::Base(Err("error"));
    let collected_option = option.collect_result();
    assert_eq!(collected_option, Err("error"));
    let option: PresentedOption<Result<i32, ()>> = PresentedOption::Choice(vec![Ok(1), Ok(2)]);
    let collected_option = option.collect_result();
    assert_eq!(collected_option, Ok(PresentedOption::Choice(vec![1, 2])));
    let option = PresentedOption::Choice(vec![Ok(1), Err("error")]);
    let collected_option = option.collect_result();
    assert_eq!(collected_option, Err("error"));
}
