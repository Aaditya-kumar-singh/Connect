use ybm_connect::groups::validation;

#[test]
fn group_name_and_description_limits() {
    assert!(validation::validate_name("Project Team").is_ok());
    assert!(validation::validate_name("ab").is_err());
    assert!(validation::validate_name(&"a".repeat(101)).is_err());
    assert!(validation::validate_description(Some(&"a".repeat(500))).is_ok());
    assert!(validation::validate_description(Some(&"a".repeat(501))).is_err());
}

#[test]
fn group_member_limits() {
    assert!(validation::validate_members(1).is_err());
    assert!(validation::validate_members(2).is_ok());
    assert!(validation::validate_members(256).is_ok());
    assert!(validation::validate_members(257).is_err());
}

#[test]
fn group_roles_are_restricted() {
    assert!(validation::validate_role("ADMIN").is_ok());
    assert!(validation::validate_role("MEMBER").is_ok());
    assert!(validation::validate_role("OWNER").is_err());
}
