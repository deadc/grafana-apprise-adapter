# grafana-apprise-adapter
Send [grafana](https://grafana.com/docs/grafana/latest/alerting/notifications/) alerts to [apprise](https://github.com/caronc/apprise) for notifications

![CI](https://github.com/RealOrangeOne/grafana-apprise-adapter/workflows/CI/badge.svg)

## Configuration

- `$APPRISE_URL`: Base URL for [apprise API](https://github.com/caronc/apprise-api/). **required**
- `$PORT`: Port to listen on, defaults to `5000`
- `$WORKERS`: Worker processes to run. Defaults to 1. If you need more, you might be doing something wrong.
- `$APPEND_LABELS`: Whether to append common labels to the alert body (Unified Alerting only). Defaults to `true`.
- `$APPEND_ANNOTATIONS`: Whether to append common annotations to the alert body (Unified Alerting only). Defaults to `true`.
- `$TITLE_ANNOTATION`: The name of the annotation to use for the notification title (Unified Alerting only).
- `$BODY_ANNOTATION`: The name of the annotation to use for the notification body (Unified Alerting only).

All of these can be overridden on a per-request basis using query parameters:

- `append_labels`: `true` or `false`
- `append_annotations`: `true` or `false`
- `title_annotation`: The name of the annotation
- `body_annotation`: The name of the annotation
