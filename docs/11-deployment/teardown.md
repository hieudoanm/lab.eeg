## 17. Teardown and Resource Cleanup

Provide a documented teardown process for disposable environments.

Before deleting resources:

1. Identify the environment.
2. Confirm that it contains no data that must be retained.
3. Check for storage objects, database backups, and Terraform state.
4. Review the destruction plan.
5. Confirm that shared resources will not be removed.
6. Apply the approved teardown.
7. Verify that the intended resources were removed.
8. Check for remaining billable resources.

Do not automatically destroy production resources as part of an ordinary deployment failure handler.

Object storage, backups, container images, logging, networking, and managed databases may have independent retention or billing behaviour. Verify each service explicitly.

