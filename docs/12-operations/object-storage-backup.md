## Object Storage Backup

Recordings are stored in object storage (local filesystem or GCS). Backups should include:

### Local Filesystem Adapter

```bash
# Sync to backup location
rsync -av --delete /var/lib/eeg-lab/uploads/ /backup/eeg-lab/uploads/
```

### Google Cloud Storage

Enable **Object Versioning** on the bucket:

```bash
gsutil versioning set on gs://eeg-lab-uploads
```

Set a **lifecycle rule** to delete old versions after 90 days.

