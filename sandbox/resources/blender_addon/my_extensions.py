bl_info = {
    "name": "Ashiojin Engine Tools (Mesh Extension)",
    "author": "Ashiojin",
    "version": (1, 3),
    "blender": (4, 0, 0),
    "location": "Properties > Material & Properties > Object",
    "description": "Custom glTF extensions for Bevy (Materials & glTF Mesh level).",
    "category": "Import-Export",
}

import bpy

# ====================================================================
# 1. MATERIAL EXTENSION DATA STRUCTURES
# ====================================================================

class AshiojinSandboxShaderParameters(bpy.types.PropertyGroup):
    param1: bpy.props.FloatVectorProperty(
        name="param1(RGBA)",
        description="Shader param1",
        size=4,
        subtype="COLOR",
        min=0,
        max=1,
        default=(1.0, 1.0, 1.0, 1.0)
    )

# ====================================================================
# 2. PROPERTY REGISTRATION (Material & Object)
# ====================================================================

def register_properties():
    bpy.utils.register_class(AshiojinSandboxShaderParameters)
    
    # Material Properties
    bpy.types.Material.ashiojin_shader_type = bpy.props.EnumProperty(
        name="Custom Shader Type",
        items=[
            ('None', "None", "Use StandardMaterial"),
            ('Sandbox', "Ashiojin Sandbox", "Use App defined Material of learn_shader.sandbox"),
            ('Sandbox_sub', "Ashiojin Sandbox", "Use App defined Another Material of learn_shader.sandbox")
        ],
        default='None'
    )
    bpy.types.Material.ashiojin_sandbox_param = bpy.props.PointerProperty(type=AshiojinSandboxShaderParameters)
    
    # Object Properties for FX (Checked via Mesh Context during Export)
    bpy.types.Mesh.ashiojin_is_fx_mesh = bpy.props.BoolProperty(
        name="Is FX Mesh",
        description="Mark this mesh as an effect tracking guide for the engine",
        default=False
    )
    bpy.types.Mesh.ashiojin_fx_type = bpy.props.EnumProperty(
        name="FX Type",
        description="Select the category of the effect",
        items=[
            ('Trail', "Trail / Ribbon", "Generates a trail following the vertices"),
            ('ParticleEmitter', "Particle Emitter", "Spawns particles at vertex positions"),
            ('Beam', "Laser / Beam", "Draws a beam between vertices")
        ],
        default='Trail'
    )

def unregister_properties():
    del bpy.types.Mesh.ashiojin_fx_type
    del bpy.types.Mesh.ashiojin_is_fx_mesh
    del bpy.types.Material.ashiojin_sandbox_param
    del bpy.types.Material.ashiojin_shader_type
    bpy.utils.unregister_class(AshiojinSandboxShaderParameters)


# ====================================================================
# 3. UI PANELS
# ====================================================================

class PANEL_PT_my_engine_settings(bpy.types.Panel):
    bl_label = "My Engine Material Settings"
    bl_idname = "PANEL_PT_my_engine_settings"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "material"

    def draw(self, context):
        layout = self.layout
        material = context.material
        
        if not material:
            layout.label(text="No Active Material")
            return

        layout.prop(material, "ashiojin_shader_type")
        if material.ashiojin_shader_type == 'Sandbox' or material.ashiojin_shader_type == 'Sandbox_sub':
            params = material.ashiojin_sandbox_param
            layout.prop(params, "param1")


class PANEL_PT_my_engine_object_settings(bpy.types.Panel):
    bl_label = "My Engine Object Settings"
    bl_idname = "PANEL_PT_my_engine_object_settings"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "data"

    def draw(self, context):
        layout = self.layout
        mesh = context.mesh

        layout.prop(mesh, "ashiojin_is_fx_mesh")
        
        if mesh.ashiojin_is_fx_mesh:
            box = layout.box()
            box.prop(mesh, "ashiojin_fx_type")


# ====================================================================
# 4. REGISTRATION BOILERPLATE
# ====================================================================

classes = (
    PANEL_PT_my_engine_settings,
    PANEL_PT_my_engine_object_settings,
)

fn_register, fn_unregister = bpy.utils.register_classes_factory(classes)

def register():
    register_properties()
    fn_register()

def unregister():
    fn_unregister()
    unregister_properties()


# ====================================================================
# 5. GLTF EXPORTER EXTENSION HOOKS
# ====================================================================

class glTF2ExportUserExtension:
    def __init__(self):
        from io_scene_gltf2.io.com.gltf2_io_extensions import Extension
        self.Extension = Extension

    # --- Material Hook ---
    def gather_material_hook(self, gltf2_material, blender_material, export_settings):
        if not hasattr(blender_material, "ashiojin_shader_type"):
            return
        if blender_material.ashiojin_shader_type == 'None':
            return

        if blender_material.ashiojin_shader_type == 'Sandbox' or blender_material.ashiojin_shader_type == 'Sandbox_sub':
            params = blender_material.ashiojin_sandbox_param
            rgba_list = list(params.param1)

            if blender_material.ashiojin_shader_type == 'Sandbox':
                extension_payload = {
                    "shader_type": "ASHIOJIN_SANDBOX",
                    "param1": rgba_list
                }
            else:
                extension_payload = {
                    "shader_type": "ASHIOJIN_SANDBOX_SUB",
                    "param1": rgba_list
                }

            if gltf2_material.extensions is None:
                gltf2_material.extensions = {}

            extension_name = "ASHIOJIN_material_sandbox"
            gltf2_material.extensions[extension_name] = self.Extension(
                name=extension_name,
                extension=extension_payload,
                required=False
            )    

    # --- Mesh Hook (Changed from Node Hook) ---
    def gather_mesh_hook(self, gltf2_mesh, blender_mesh, blender_object, vertex_groups, modifiers, materials, export_settings):
        """
        Triggers for every Mesh data block being exported into the glTF file.
        In Blender, curves are automatically evaluated/converted to meshes during export.
        """
        # export_settings から現在処理中の元のBlenderオブジェクトを取得
        # (カーブがメッシュ化された場合でも元のオブジェクトのカスタムプロパティを参照可能)
        #blender_object = export_settings.get('gltf_current_object')
        #if not blender_object:
        #    print(f"!1")
        #    return

        if not hasattr(blender_mesh, "ashiojin_is_fx_mesh"):
            print(f"not ashiojin_is_fx_mesh")
            return

        if not blender_mesh.ashiojin_is_fx_mesh:
            print(f"not ablender_object.ashiojin_is_fx_mesh")
            return

        # Construct payload with target identifiers
        extension_payload = {
            "is_fx_mesh": True,
            "fx_type": blender_mesh.ashiojin_fx_type
        }

        if gltf2_mesh.extensions is None:
            gltf2_mesh.extensions = {}

        # Custom namespace for mesh level metadata
        extension_name = "ASHIOJIN_mesh_fx_config"
        gltf2_mesh.extensions[extension_name] = self.Extension(
            name=extension_name,
            extension=extension_payload,
            required=False
        )
        print(f"Exported FX metadata attached to MESH data: {blender_mesh.name}")


if __name__ == "__main__":
    register()