#[tokio::test]
#[ignore = "requires PostgreSQL, Redis, authenticated HTTP clients, and live group/message flow"]
async fn group_flow_covers_create_membership_roles_leave_remove_and_messages() {
    // Live integration coverage belongs here once the service-backed test environment is available.
}
