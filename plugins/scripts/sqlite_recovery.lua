-- SQLite database structure validator
--
-- Validates SQLite page headers to confirm database integrity.
-- Critical for recovering WhatsApp, Signal, browser history,
-- and other app databases from mobile device images.

signature {
    name     = "SQLite Database (Validated)",
    category = "Databases",
    header   = "SQLite format 3\000",
    max_size = "2GB",

    validate = function(data)
        if #data < 100 then
            return false
        end

        -- Page size at offset 16-17 (big-endian)
        local page_size = (data:byte(17) * 256) + data:byte(18)

        -- Valid page sizes: powers of 2 between 512 and 65536
        if page_size < 512 or page_size > 65536 then
            return false
        end

        -- Check power of 2 using bitwise AND
        if (page_size % (page_size - 1)) ~= 0 then
            -- Fallback: manual power-of-2 check
            local valid_sizes = {512, 1024, 2048, 4096, 8192, 16384, 32768, 65536}
            local found = false
            for _, v in ipairs(valid_sizes) do
                if page_size == v then found = true; break end
            end
            if not found then return false end
        end

        -- File format write version at offset 18 (should be 1 or 2)
        local write_ver = data:byte(19)
        return write_ver == 1 or write_ver == 2
    end
}
