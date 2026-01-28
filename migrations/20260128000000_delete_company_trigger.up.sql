CREATE TRIGGER delete_company_after_application
AFTER DELETE ON applications
BEGIN
    DELETE FROM companies WHERE id = OLD.company_id;
END;
