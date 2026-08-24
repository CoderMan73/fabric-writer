package __PACKAGE__;

import net.fabricmc.api.ModInitializer;

import net.minecraft.resources.Identifier;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class __MOD_CLASS__ implements ModInitializer {
    public static final String MOD_ID = "__MOD_ID__";

    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    @Override
    public void onInitialize() {
        LOGGER.info("Hello Fabric world!");
        // ============================================
        // FABRIC-WRITER — MANAGED SECTION
        // ============================================
        ModItems.initialize();
        // ============================================
        // END FABRIC-WRITER MANAGED
        // ============================================
    }

    public static Identifier id(String path) {
        return Identifier.fromNamespaceAndPath(MOD_ID, path);
    }
}
