"""Mnemix adapter for CI-bot workflows."""

from __future__ import annotations

from ._adapter_base import BaseAdapter, CiRunContext


class CiBotAdapter(BaseAdapter):
    """Workflow helpers for CI bots and automated runbooks."""

    def prepare_run(
        self,
        *,
        scope: str,
        run_id: str,
        pipeline: str,
        limit: int = 8,
        create_checkpoint: bool = True,
    ) -> CiRunContext:
        checkpoint = None
        if create_checkpoint:
            checkpoint = self.create_checkpoint(
                f"ci-{run_id}",
                description=f"Checkpoint before CI pipeline {pipeline}",
            )
        bundle = self._context_bundle(
            scope=scope,
            query=pipeline,
            heading="Relevant CI memory for this run:",
            limit=limit,
        )
        return CiRunContext(bundle=bundle, checkpoint=checkpoint)

    def record_failure(
        self,
        *,
        memory_id: str,
        scope: str,
        title: str,
        summary: str,
        detail: str,
    ):
        return self._remember(
            memory_id=memory_id,
            scope=scope,
            kind="warning",
            title=title,
            summary=summary,
            detail=detail,
            importance=90,
            tags=["ci", "failure"],
            source_tool="ci-bot",
        )

    def record_fix(
        self,
        *,
        memory_id: str,
        scope: str,
        title: str,
        summary: str,
        detail: str,
    ):
        return self._remember(
            memory_id=memory_id,
            scope=scope,
            kind="procedure",
            title=title,
            summary=summary,
            detail=detail,
            importance=85,
            tags=["ci", "fix"],
            source_tool="ci-bot",
        )
