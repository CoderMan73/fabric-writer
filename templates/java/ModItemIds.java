package __PACKAGE__;

import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.item.Item;

public class ModItemIds {
    public static ResourceKey<Item> create(String name) {
        return ResourceKey.create(Registries.ITEM, Identifier.fromNamespaceAndPath(__MOD_CLASS__.MOD_ID, name));
    }

    // ============================================
    // FABRIC-WRITER — MANAGED SECTION
    // ============================================
    __MANAGED_CONTENT__
    // ============================================
    // END FABRIC-WRITER MANAGED
    // ============================================
}
