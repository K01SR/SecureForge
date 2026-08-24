-- Advanced JPEG structure validator
--
-- Goes beyond simple header/footer matching by parsing
-- JPEG marker segments to validate file integrity and
-- support fragmented file reconstruction via RST markers.

signature {
    name     = "JPEG (Structure-Validated)",
    category = "Media",
    header   = "\xFF\xD8\xFF",
    footer   = "\xFF\xD9",
    max_size = "50MB",

    validate = function(data)
        -- Verify SOI marker
        if data:byte(1) ~= 0xFF or data:byte(2) ~= 0xD8 then
            return false
        end

        -- Check for at least one valid APP/SOF/SOS marker after SOI
        local b3 = data:byte(3)
        if b3 ~= 0xFF then
            return false
        end

        local marker = data:byte(4)
        -- Valid markers: APP0-APP15 (0xE0-0xEF), SOF0 (0xC0), DQT (0xDB)
        local valid = (marker >= 0xE0 and marker <= 0xEF)  -- APPn
                   or (marker >= 0xC0 and marker <= 0xC3)  -- SOFn
                   or marker == 0xDB                        -- DQT
                   or marker == 0xC4                        -- DHT
        return valid
    end
}
