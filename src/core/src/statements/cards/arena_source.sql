SELECT json_object('arena_id', c.GrpId, 'name', l.Loc,
    'set_code', lower(c.ExpansionCode),
    'collector_number', c.CollectorNumber || coalesce(c.CollectorSuffix, ''))
FROM Cards c JOIN Localizations_enUS l ON l.LocId=c.TitleId AND l.Formatted=1
WHERE l.Loc IS NOT NULL
ORDER BY c.GrpId
