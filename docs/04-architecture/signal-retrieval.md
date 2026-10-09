## 9. Signal Retrieval Architecture

The frontend requests a bounded recording interval.

Example request:

```http
GET /api/v1/organisations/{organisation_id}/datasets/{dataset_id}/signals?start_seconds=10&end_seconds=15&channels=0,1
```

The API should:

1. Authenticate the request.
2. Verify organisation and dataset access.
3. Verify that the dataset is ready.
4. Validate the time interval and channel selection.
5. Enforce maximum request and response sizes.
6. Read only the required portion when supported by the chosen file representation.
7. Apply documented display downsampling if necessary.
8. Return the signal samples and corresponding time axis.
9. Include metadata that explains the response representation.

### Performance considerations

A basic `.npy` file is not automatically an efficient format for every partial-read workload.

If the selected format requires loading a large file for each interval, use memory mapping where supported, bounded caching, or a later storage-format change. Do not allow arbitrary interval requests to exhaust worker or API memory.

The API should avoid sending an entire recording to the browser simply because the user opened the explorer.

