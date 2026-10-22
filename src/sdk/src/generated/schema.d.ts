export interface paths {
    "/v1/capture": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Read durable capture counts. Unknown results include unfinished and ambiguous games. */
        get: operations["captureStatusGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/cards/resolve-arena": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Resolve Arena IDs in one batch, preserving unknowns and mapping provenance. */
        post: operations["cardsResolveArena"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/cards/resolve-references": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Resolve mixed Oracle, Scryfall-printing and Arena references in one batch. */
        post: operations["cardsResolveReferences"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/cards/search": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Search the offline catalog by text and structured filters. */
        post: operations["cardsSearch"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/cards/{oracle_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Read complete original Scryfall metadata, including card faces. */
        get: operations["cardsGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/catalog": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Read catalog version and offline availability. */
        get: operations["catalogStatusGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/catalog/refresh": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Refresh public Scryfall metadata while preserving the prior offline catalog on failure. */
        post: operations["catalogRefresh"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/deck-versions": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Append a deck version; earlier versions stay immutable. */
        post: operations["deckVersionCreate"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/deck-versions/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Read an immutable deck version and its original card references. */
        get: operations["deckVersionGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/deck-versions/{id}/analysis": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /**
         * Analyze an immutable deck version using the current local catalog. Printed
         *     curves remain available when missing or unsupported rules block simulation.
         */
        get: operations["analysisProfileGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/deck-versions/{id}/arena": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Export a saved version for pasting into Arena. */
        get: operations["deckArenaExport"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/decks": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List local decks with stable cursor pagination. */
        get: operations["decksList"];
        put?: never;
        /** Create a deck and its first immutable version. */
        post: operations["decksCreate"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/decks/import-arena": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** Import Arena text, reporting every unresolved card instead of dropping lines. */
        post: operations["deckArenaImport"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/discovery": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Discover implemented local capabilities and stable operation identifiers. */
        get: operations["systemDiscovery"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/games": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List captured games with immutable deck attribution, samples and missing-data flags. */
        get: operations["gamesList"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/games/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Read one game; known outcomes do not imply a complete timeline. */
        get: operations["gameGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/games/{id}/events": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Read a stable, bounded event page. Keep returned cursors opaque; later captures do not alter this pagination snapshot. */
        get: operations["gameTimeline"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/games/{id}/turns": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Group observed events into turns without inferring absent actions. Pagination freezes capture evidence; unassigned events are counted separately. */
        get: operations["gameTurns"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/health": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Read the local service build identity. */
        get: operations["healthGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/matches": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List matches separately from their games. Deck/version filters select matches containing at least one matching game. */
        get: operations["matchesList"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/matches/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Read a match and its games. Match outcomes require an explicit match-scoped result. */
        get: operations["matchGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/notes": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List notes without changing captured history. */
        get: operations["notesList"];
        put?: never;
        /** Add an idempotent local note alongside a deck or version. */
        post: operations["notesCreate"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/probabilities/draw": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /**
         * Exact finite-deck draw probability without replacement. This is not a
         *     battlefield, tutor, mulligan or Arena smoothing model.
         */
        post: operations["probabilityDraw"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/probabilities/joint": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /**
         * Exact joint draw probability across mutually exclusive categories covering
         *     the whole deck. Split overlapping groups into intersection categories first.
         */
        post: operations["probabilityJoint"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/simulations": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** List persisted simulation jobs in stable creation order using opaque cursors. */
        get: operations["simulationsList"];
        put?: never;
        /**
         * Queue a seeded local simulation. Inputs and model versions are frozen before
         *     execution; retries with the same key return the original job.
         */
        post: operations["simulationsCreate"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/simulations/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Read durable progress and results. A missing report means no completed trials. */
        get: operations["simulationGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/simulations/{id}/cancel": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /**
         * Request cancellation. Bounded batches preserve completed-trial denominators;
         *     cancelling an already terminal job is an idempotent read of its final state.
         */
        post: operations["simulationCancel"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/simulations/{id}/inputs": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Read the frozen card metadata and parameters used by a job, not today's catalog. */
        get: operations["simulationInputsGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/v1/statistics": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** Observe game-level performance with explicit denominators. Unknowns are excluded from decisive win rate; recorded land counters cover only sampled local turns. */
        get: operations["statisticsGet"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        ArenaExport: {
            text: string;
        };
        ArenaIds: {
            ids: number[];
        };
        ArenaResolution: {
            /** Format: int32 */
            arena_id: number;
            /** @description The original printing metadata; null explicitly means unresolved. */
            card?: unknown;
            resolved_by?: string | null;
        };
        BuildInfo: {
            name: string;
            version: string;
        };
        CaptureStatus: {
            /** Format: int64 */
            deck_versions: number;
            /** Format: int64 */
            events: number;
            /** Format: int64 */
            games: number;
            /** Format: int64 */
            matches: number;
            /** Format: int64 */
            sources: number;
            /**
             * Format: int64
             * @description Games without one unambiguous observed local outcome, including unfinished games.
             */
            unknown_results: number;
        };
        CardPage: {
            items: components["schemas"]["CardSummary"][];
            next_cursor?: string | null;
        };
        CardProfile: {
            adventure_cost?: string | null;
            cast_cost?: string | null;
            equip_cost?: string | null;
            issues: components["schemas"]["Issue"][];
            name?: string | null;
            oracle_id?: string | null;
            /** Format: int32 */
            quantity: number;
            reference: string;
            zone: components["schemas"]["Zone"];
        };
        CardSummary: {
            arena_available: boolean;
            color_identity: string[];
            id: string;
            image_uri?: string | null;
            mana_cost?: string | null;
            /** Format: double */
            mana_value?: number | null;
            name: string;
            oracle_id: string;
            oracle_text: string;
            type_line: string;
        };
        CatalogSource: {
            updated_at: string;
            url: string;
        };
        CatalogStatus: {
            /** Format: int64 */
            arena_ids: number;
            /** Format: int64 */
            cards: number;
            /** Format: int64 */
            printings: number;
            snapshot_id?: string | null;
            source?: null | components["schemas"]["CatalogSource"];
        };
        Category: {
            label: string;
            /** Format: int32 */
            max: number;
            /** Format: int32 */
            min: number;
            /** Format: int32 */
            population: number;
        };
        Counts: {
            /** Format: int32 */
            creatures: number;
            /** Format: int32 */
            equipment: number;
            /** Format: int32 */
            lands: number;
            /** Format: int32 */
            legendary: number;
            /** Format: int32 */
            main: number;
            /** Format: int32 */
            sideboard: number;
            /** Format: int32 */
            unresolved_main: number;
        };
        CreateDeck: {
            cards: components["schemas"]["DeckCard"][];
            format: string;
            name: string;
        };
        CreateDeckCommand: {
            idempotency_key: string;
            input: components["schemas"]["CreateDeck"];
        };
        CreateNoteCommand: {
            idempotency_key: string;
            input: components["schemas"]["NewNote"];
        };
        CreateSimulationCommand: {
            idempotency_key: string;
            input: components["schemas"]["SimulationParameters"];
        };
        CreateVersionCommand: {
            idempotency_key: string;
            input: components["schemas"]["NewVersion"];
        };
        CurveBin: {
            /** Format: double */
            mana_value?: number | null;
            /** Format: int32 */
            quantity: number;
        };
        DeckCard: {
            /** Format: int32 */
            quantity: number;
            /** @description oracle:<id>, scryfall:<printing-id>, or arena:<numeric-id>. */
            reference: string;
            zone: components["schemas"]["Zone"];
        };
        DeckPage: {
            items: components["schemas"]["DeckSummary"][];
            next_cursor?: string | null;
        };
        DeckSnapshot: {
            cards: components["schemas"]["ResolvedCard"][];
            catalog_snapshot?: string | null;
            catalog_updated_at?: string | null;
            counts: components["schemas"]["Counts"];
            curve: components["schemas"]["CurveBin"][];
            deck_id: string;
            format: string;
            name: string;
            version_id: string;
        };
        DeckSummary: {
            format: string;
            id: string;
            latest_version: string;
            /** Format: int32 */
            main_count: number;
            name: string;
            /** Format: int32 */
            version_count: number;
        };
        DeckVersion: {
            cards: components["schemas"]["DeckCard"][];
            /** Format: int64 */
            created_at: number;
            deck_id: string;
            format: string;
            id: string;
            name: string;
            /** Format: int32 */
            ordinal: number;
            origin: components["schemas"]["Origin"];
            parent_version?: string | null;
        };
        Discovery: {
            capabilities: string[];
            local_only: boolean;
            name: string;
            operations: string[];
            version: string;
        };
        DrawProbability: {
            /** Format: int32 */
            draws: number;
            /** Format: int32 */
            max: number;
            /** Format: int32 */
            min: number;
            /** Format: int32 */
            population: number;
            /** Format: int32 */
            successes: number;
        };
        /** @enum {string} */
        ErrorCode: "unauthorized" | "forbidden" | "invalid_request" | "not_found" | "conflict" | "unresolved_cards" | "unsupported_model" | "busy" | "internal";
        /** @description Wilson score interval, with explicit numerator and denominator. */
        Estimate: {
            interval95: number[];
            /** Format: double */
            probability: number;
            /** Format: int32 */
            samples: number;
            /** Format: int32 */
            successes: number;
        };
        ExactProbability: {
            /** Format: int32 */
            draws: number;
            method: components["schemas"]["ProbabilityMethod"];
            /** Format: int32 */
            population: number;
            /** Format: double */
            probability: number;
        };
        GamePage: {
            items: components["schemas"]["GameSummary"][];
            next_cursor?: string | null;
        };
        GameSummary: {
            capture_complete: boolean;
            deck_id?: string | null;
            deck_name?: string | null;
            deck_version?: string | null;
            /** Format: int64 */
            events: number;
            /** Format: int64 */
            first_ordinal: number;
            format?: string | null;
            id: string;
            /**
             * Format: int64
             * @description Recorded land counters on observed local turns; not an inferred full-game total.
             */
            land_drops_observed?: number | null;
            /** Format: int64 */
            land_turn_samples: number;
            /** Format: int32 */
            last_turn?: number | null;
            /** Format: int64 */
            local_seat?: number | null;
            /** @description Zone-less Arena log clock; not silently interpreted as UTC. */
            local_time?: string | null;
            match_id: string;
            /** @description Explicit match clock evidence; not a reconstructed game-start timestamp. */
            match_instant_utc?: string | null;
            missing_data: components["schemas"]["MissingData"][];
            mode: components["schemas"]["MatchMode"];
            /** Format: int64 */
            mulligan_actions_observed: number;
            /** Format: int32 */
            mulligans?: number | null;
            /** Format: int32 */
            number: number;
            on_play?: boolean | null;
            outcome?: null | components["schemas"]["ObservedOutcome"];
            /** Format: int64 */
            turns_observed: number;
        };
        Goal: {
            attached: number;
            attack_ready: number;
            equipment: number;
        };
        ImportDeck: {
            format: string;
            name: string;
            text: string;
        };
        ImportDeckCommand: {
            idempotency_key: string;
            input: components["schemas"]["ImportDeck"];
        };
        Issue: {
            blocking: boolean;
            code: components["schemas"]["IssueCode"];
            detail: string;
        };
        /** @enum {string} */
        IssueCode: "missing_card" | "invalid_metadata" | "unsupported_cost" | "unknown_rules" | "rules_changed" | "unsupported_layout" | "combat_excluded" | "recursion_excluded" | "activated_ability_excluded" | "ignored_card";
        JobPage: {
            items: components["schemas"]["SimulationJob"][];
            next_cursor?: string | null;
        };
        /** @enum {string} */
        JobStatus: "queued" | "running" | "cancelling" | "completed" | "cancelled" | "failed" | "interrupted";
        JointProbability: {
            /**
             * @description Mutually exclusive categories covering the whole deck, including other cards.
             *     Split overlapping groups into explicit intersections before using this model.
             */
            categories: components["schemas"]["Category"][];
            /** Format: int32 */
            draws: number;
        };
        /** @enum {string} */
        MatchMode: "bo1" | "bo3" | "unknown";
        MatchPage: {
            items: components["schemas"]["MatchSummary"][];
            next_cursor?: string | null;
        };
        MatchSummary: {
            completed: boolean;
            /** Format: int64 */
            first_ordinal: number;
            /** Format: int64 */
            game_count: number;
            games: components["schemas"]["GameSummary"][];
            id: string;
            mode: components["schemas"]["MatchMode"];
            outcome?: null | components["schemas"]["ObservedOutcome"];
        };
        /** @enum {string} */
        MissingData: "initial_state" | "terminal_result" | "state_gap" | "unverifiable_chain" | "truncated" | "connection" | "deck_version" | "local_seat" | "conflicting_result" | "outcome";
        /** @enum {string} */
        ModelScope: "precombat_goldfish";
        MulliganParameters: {
            /** Format: int32 */
            force_mulligans: number;
            /** Format: int32 */
            max_lands: number;
            /** Format: int32 */
            min_lands: number;
            /** Format: int32 */
            minimum_keep: number;
            required_references: string[];
        };
        NewNote: {
            body: string;
            deck_id: string;
            version_id?: string | null;
        };
        NewVersion: {
            cards: components["schemas"]["DeckCard"][];
            deck_id: string;
            format: string;
            name: string;
            parent_version?: string | null;
        };
        Note: {
            body: string;
            /** Format: int64 */
            created_at: number;
            deck_id: string;
            id: string;
            origin: components["schemas"]["Origin"];
            version_id?: string | null;
        };
        NotePage: {
            items: components["schemas"]["Note"][];
            next_cursor?: string | null;
        };
        /** @enum {string} */
        ObservedOutcome: "won" | "lost" | "draw";
        /** @enum {string} */
        Origin: "user" | "agent" | "arena" | "import";
        /** @enum {string} */
        ProbabilityMethod: "hypergeometric" | "multivariate_hypergeometric";
        Problem: {
            code: components["schemas"]["ErrorCode"];
            message: string;
            unresolved?: string[] | null;
        };
        Profile: {
            assumptions: string[];
            can_simulate: boolean;
            cards: components["schemas"]["CardProfile"][];
            catalog_snapshot?: string | null;
            catalog_updated_at?: string | null;
            compiler_version: string;
            counts: components["schemas"]["Counts"];
            curve: components["schemas"]["CurveBin"][];
            deck_id: string;
            format: string;
            model_version: string;
            name: string;
            policy_version: string;
            sources: components["schemas"]["SourceCount"][];
            version_id: string;
        };
        ReferenceResolution: {
            card?: unknown;
            reference: string;
        };
        References: {
            references: string[];
        };
        ResolvedCard: {
            data?: unknown;
            /** Format: int32 */
            quantity: number;
            reference: string;
            zone: components["schemas"]["Zone"];
        };
        SampleBreakdown: {
            /** Format: int64 */
            draws: number;
            /** Format: int64 */
            games: number;
            /** Format: int64 */
            known_results: number;
            /** Format: int64 */
            losses: number;
            mode: components["schemas"]["MatchMode"];
            /** Format: int64 */
            wins: number;
        };
        Search: {
            arena_only?: boolean;
            colors?: string | null;
            cursor?: string | null;
            format?: string | null;
            legendary?: boolean | null;
            limit?: number;
            /** Format: double */
            mana_max?: number | null;
            /** Format: double */
            mana_min?: number | null;
            /** @description Plain text tokens across names, rules text and types. Not raw FTS syntax. */
            text?: string;
            type_line?: string | null;
        };
        SimulationInputs: {
            parameters: components["schemas"]["SimulationParameters"];
            profile: components["schemas"]["Profile"];
            snapshot: components["schemas"]["DeckSnapshot"];
        };
        SimulationJob: {
            cancellation_origin?: null | components["schemas"]["Origin"];
            catalog_snapshot?: string | null;
            compiler_version: string;
            /** Format: int32 */
            completed_runs: number;
            /** Format: int64 */
            created_at: number;
            error_code?: string | null;
            id: string;
            model_version: string;
            origin: components["schemas"]["Origin"];
            parameters: components["schemas"]["SimulationParameters"];
            policy_version: string;
            replay_version: string;
            report?: null | components["schemas"]["SimulationReport"];
            /** Format: int32 */
            requested_runs: number;
            seed: string;
            status: components["schemas"]["JobStatus"];
            /** Format: int64 */
            updated_at: number;
            version_id: string;
        };
        SimulationParameters: {
            goal: components["schemas"]["Goal"];
            mulligan: components["schemas"]["MulliganParameters"];
            on_the_play: boolean;
            /** Format: int32 */
            runs: number;
            scope: components["schemas"]["ModelScope"];
            /** @description Full-width hexadecimal seed. Omit to generate and persist a random seed. */
            seed?: string | null;
            /** Format: int32 */
            turns: number;
            version_id: string;
        };
        SimulationReport: {
            /** Format: int32 */
            completed_runs: number;
            /** Format: int32 */
            forced_keeps: number;
            goal: components["schemas"]["Goal"];
            /** Format: double */
            mean_mulligans: number;
            /** Format: double */
            mean_opening_size: number;
            model_version: string;
            policy: string;
            replay_version: string;
            /** Format: int32 */
            requested_runs: number;
            seed: string;
            turns: components["schemas"]["TurnEstimate"][];
        };
        SourceCount: {
            color: string;
            /**
             * Format: int32
             * @description Potential land color depends on board state or spending restriction.
             */
            conditional_lands: number;
            /**
             * Format: int32
             * @description Includes summoning-sick or restricted creature/artifact mana abilities.
             */
            nonland_sources: number;
            /**
             * Format: int32
             * @description Color always available once untapped, with no spending restriction.
             */
            unrestricted_lands: number;
        };
        Statistics: {
            by_mode: components["schemas"]["SampleBreakdown"][];
            /** Format: int64 */
            decisive_games: number;
            /**
             * Format: double
             * @description Wins / (wins + losses), excluding draws and unknown results.
             */
            decisive_win_rate?: number | null;
            /** Format: int64 */
            draws: number;
            /** Format: int64 */
            games: number;
            /** Format: int64 */
            known_results: number;
            /** Format: int64 */
            land_turn_samples: number;
            /** Format: int64 */
            losses: number;
            /** Format: int64 */
            matches: number;
            /** Format: int64 */
            mulligan_samples: number;
            /** Format: double */
            observed_mulligans_average?: number | null;
            /** Format: int64 */
            on_draw_samples: number;
            /** Format: int64 */
            on_play_samples: number;
            /** Format: int64 */
            partial_games: number;
            /** Format: int64 */
            recorded_land_drops: number;
            /** Format: int64 */
            unknown_play_draw: number;
            /** Format: int64 */
            unknown_results: number;
            /** Format: int64 */
            wins: number;
        };
        TimelineEvent: {
            data: {
                [key: string]: unknown;
            };
            id: string;
            instant_utc?: string | null;
            kind: string;
            /** Format: int64 */
            local_seat?: number | null;
            local_time?: string | null;
            /** Format: int64 */
            ordinal: number;
            phase?: string | null;
            /** Format: int64 */
            state_id?: number | null;
            step?: string | null;
            /** Format: int32 */
            turn_number?: number | null;
        };
        TimelinePage: {
            game_id: string;
            items: components["schemas"]["TimelineEvent"][];
            next_cursor?: string | null;
            /** Format: int64 */
            through_ordinal: number;
        };
        TurnEstimate: {
            attached: components["schemas"]["Estimate"];
            attack_ready: components["schemas"]["Estimate"];
            /** Format: int32 */
            decked: number;
            equipment: components["schemas"]["Estimate"];
            joint: components["schemas"]["Estimate"];
            /** Format: double */
            mean_cards_discarded: number;
            /** Format: double */
            mean_cards_drawn: number;
            /** Format: double */
            mean_creatures: number;
            /** Format: double */
            mean_equipment: number;
            /** Format: double */
            mean_legendary_bricks: number;
            /** Format: double */
            mean_life: number;
            /** Format: double */
            mean_tutors_cast: number;
            /** Format: int32 */
            story_samples: number;
            /** Format: int32 */
            turn: number;
        };
        TurnPage: {
            game_id: string;
            items: components["schemas"]["TurnSummary"][];
            next_cursor?: string | null;
            /** Format: int64 */
            through_ordinal: number;
            /** Format: int64 */
            unassigned_events: number;
        };
        TurnSummary: {
            /** Format: int64 */
            actions: number;
            /** Format: int64 */
            active_player?: number | null;
            /** Format: int64 */
            events: number;
            /** Format: int64 */
            first_ordinal: number;
            is_local_turn?: boolean | null;
            /** Format: int64 */
            land_counter_samples: number;
            /** Format: int64 */
            land_drops_observed?: number | null;
            /** Format: int64 */
            last_ordinal: number;
            /** Format: int64 */
            local_life_total?: number | null;
            /** Format: int32 */
            mulligans?: number | null;
            /** Format: int32 */
            number: number;
            /** Format: int64 */
            phase_samples: number;
            phases: string[];
            /** Format: int64 */
            states: number;
        };
        /** @enum {string} */
        Zone: "main" | "sideboard" | "command" | "companion";
    };
    responses: never;
    parameters: never;
    requestBodies: never;
    headers: never;
    pathItems: never;
}
export type $defs = Record<string, never>;
export interface operations {
    captureStatusGet: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CaptureStatus"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    cardsResolveArena: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["ArenaIds"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ArenaResolution"][];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    cardsResolveReferences: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["References"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ReferenceResolution"][];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    cardsSearch: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["Search"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CardPage"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    cardsGet: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                oracle_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": {
                        [key: string]: unknown;
                    };
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    catalogStatusGet: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CatalogStatus"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    catalogRefresh: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["CatalogStatus"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    deckVersionCreate: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateVersionCommand"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DeckVersion"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    deckVersionGet: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DeckVersion"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    analysisProfileGet: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Profile"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    deckArenaExport: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ArenaExport"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    decksList: {
        parameters: {
            query?: {
                limit?: number;
                cursor?: string;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DeckPage"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    decksCreate: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateDeckCommand"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DeckVersion"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    deckArenaImport: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["ImportDeckCommand"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["DeckVersion"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    systemDiscovery: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Discovery"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    gamesList: {
        parameters: {
            query?: {
                limit?: number;
                cursor?: string;
                deck_id?: string;
                version_id?: string;
                match_id?: string;
                mode?: components["schemas"]["MatchMode"];
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GamePage"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    gameGet: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["GameSummary"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    gameTimeline: {
        parameters: {
            query?: {
                turn_number?: number;
                limit?: number;
                cursor?: string;
            };
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TimelinePage"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    gameTurns: {
        parameters: {
            query?: {
                limit?: number;
                cursor?: string;
            };
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TurnPage"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    healthGet: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description Build identity */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["BuildInfo"];
                };
            };
        };
    };
    matchesList: {
        parameters: {
            query?: {
                limit?: number;
                cursor?: string;
                deck_id?: string;
                version_id?: string;
                match_id?: string;
                mode?: components["schemas"]["MatchMode"];
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MatchPage"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    matchGet: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["MatchSummary"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    notesList: {
        parameters: {
            query: {
                deck_id: string;
                limit?: number;
                cursor?: string;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["NotePage"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    notesCreate: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateNoteCommand"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Note"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    probabilityDraw: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["DrawProbability"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExactProbability"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    probabilityJoint: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["JointProbability"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ExactProbability"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    simulationsList: {
        parameters: {
            query?: {
                limit?: number;
                cursor?: string;
                version_id?: string;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["JobPage"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    simulationsCreate: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateSimulationCommand"];
            };
        };
        responses: {
            202: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SimulationJob"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    simulationGet: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SimulationJob"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    simulationCancel: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SimulationJob"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    simulationInputsGet: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["SimulationInputs"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
    statisticsGet: {
        parameters: {
            query?: {
                deck_id?: string;
                version_id?: string;
                mode?: components["schemas"]["MatchMode"];
                complete_only?: boolean;
            };
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Statistics"];
                };
            };
            "4XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
            "5XX": {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["Problem"];
                };
            };
        };
    };
}
