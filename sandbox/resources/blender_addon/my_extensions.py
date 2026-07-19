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
# EXTENSION DATA STRUCTURES
# ====================================================================
FX_TYPE_OPTIONS = [
    ('Trail', "Trail / Ribbon", "Generates a trail following the vertices"),
    ('ParticleEmitter', "Particle Emitter", "Spawns particles at vertex positions"),
    ('Beam', "Laser / Beam", "Draws a beam between vertices")
]

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

def get_fx_mesh_items(self, context):
    items = [('NONE', 'Select a Mesh...', '')]
    for obj in context.scene.objects:
        #print(f"{obj.name}: {obj.type}")
        if obj.type == 'MESH' and obj.data.ashiojin_is_fx_mesh:
            #print(f"-- is fx mesh: ({obj.data.ashiojin_fx_type})")
            label = f"{obj.name} ({obj.data.ashiojin_fx_type})"
            items.append((obj.name, label, ""))
    return items

def update_fx_mesh_pointer(self, context):
    target_name = self.mesh_list_enum
    if target_name and target_name != 'NONE':
        target_obj = context.scene.objects.get(target_name)
        if target_obj:
            self.target_mesh = target_obj
            return
    self.target_mesh = None

class AshiojinSandboxActionConfig(bpy.types.PropertyGroup):
    mesh_list_enum: bpy.props.EnumProperty(
        name="Emitter Mesh",
        items=get_fx_mesh_items,
        update=update_fx_mesh_pointer
    )
    target_mesh: bpy.props.PointerProperty(
        type=bpy.types.Object,
        name="Tracked Mesh Data"
    )
    start_frame: bpy.props.IntProperty(name="Start Frame", default=1)
    end_frame: bpy.props.IntProperty(name="End Frame", default=2)

# TODO: Make other properties to classes


# ====================================================================
# UI List and Operator
# ====================================================================

class MY_UL_action_fx_list(bpy.types.UIList):
    """List for displaying action FX configurations in the Dopesheet panel. """
    def draw_item(self, context, layout, data, item, icon, active_data, active_propname, index):
        if self.layout_type in {'DEFAULT'}:
            box = layout.box()
            box.label(text=f"EmitTrail Parameters")
            box.prop(item, "mesh_list_enum", text="Emitter Mesh")
            if item.target_mesh:
                box.label(text=f"Tracking Live ID: {item.target_mesh.name}", icon='LINKED')
            else:
                box.label(text="No valid mesh tracked.", icon='ERROR')
            box.prop(item, "start_frame")
            box.prop(item, "end_frame")

            box.operator("action_fx_configs.delete_item", text="Delete FX Config", icon='X')
        elif self.layout_type in {'COMPACT'}:
            layout.label(text=f"{index + 1}: {item.target_mesh.name} ({item.start_frame}-{item.end_frame})")
            layout.operator("action_fx_configs.delete_item", text="Delete FX Config", icon='X')
        else:
            layout.label(text=f"FX Config {index + 1}: {item.target_mesh.name}")

class ACTION_FX_CONFIGS_OT_new_item(bpy.types.Operator):
    bl_idname = "action_fx_configs.new_item"
    bl_label = "Add a new item"

    # self.action is the Action data block that owns the collection
    def execute(self, context):
        # check self.action is valid
        action = context.object.animation_data.action

        # if not initialized, initialize the collection
        if not hasattr(action, "ashiojin_fx_configs"):
            action.ashiojin_fx_configs = bpy.props.CollectionProperty(type=AshiojinSandboxActionConfig)

        # Add a new item
        new_item = context.object.animation_data.action.ashiojin_fx_configs.add()
        # TODO: set default frame to current frame ? but we will add a feature to set start_frame/end_frame by selecting a marker or a current frame or a frame.
        return { "FINISHED" }

class ACTION_FX_CONFIGS_OT_delete_item(bpy.types.Operator):
    bl_idname = "action_fx_configs.delete_item"
    bl_label = "Delete the selected item"

    def execute(self, context):
        action = context.object.animation_data.action
        # Remove the selected item
        index = action.ashiojin_fx_configs_index
        if index >= 0 and index < len(action.ashiojin_fx_configs):
            action.ashiojin_fx_configs.remove(index)
            return { "FINISHED" }
        else:
            self.report({'ERROR'}, "Invalid index for deletion.")
            return {'CANCELLED'}

# ====================================================================
# PROPERTY REGISTRATION (Material & Object)
# ====================================================================

def register_properties():
    bpy.utils.register_class(AshiojinSandboxShaderParameters)
    
    # Material Properties
    bpy.types.Material.ashiojin_shader_type = bpy.props.EnumProperty(
        name="Custom Shader Type",
        items=[
            ('None', "None", "Use StandardMaterial"),
            ('Sandbox', "Ashiojin Sandbox", "Use App defined Material of learn_shader.sandbox"),
            ('Sandbox_sub', "Ashiojin Sandbox Sub", "Use App defined Another Material of learn_shader.sandbox")
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
        items=FX_TYPE_OPTIONS,
        default='Trail'
    )

    bpy.utils.register_class(MY_UL_action_fx_list)
    
    bpy.utils.register_class(AshiojinSandboxActionConfig)
    bpy.types.Action.ashiojin_fx_configs = bpy.props.CollectionProperty(type=AshiojinSandboxActionConfig)
    bpy.types.Action.ashiojin_fx_configs_index = bpy.props.IntProperty(name="FX Config Index", default=0)

    bpy.utils.register_class(ACTION_FX_CONFIGS_OT_new_item)
    bpy.utils.register_class(ACTION_FX_CONFIGS_OT_delete_item)


def unregister_properties():
    del bpy.types.Mesh.ashiojin_fx_type
    del bpy.types.Mesh.ashiojin_is_fx_mesh
    del bpy.types.Material.ashiojin_sandbox_param
    del bpy.types.Material.ashiojin_shader_type
    del bpy.types.Action.ashiojin_fx_configs
    del bpy.types.Action.ashiojin_fx_configs_index
    bpy.utils.unregister_class(AshiojinSandboxShaderParameters)
    bpy.utils.unregister_class(AshiojinSandboxActionConfig)
    bpy.utils.unregister_class(MY_UL_action_fx_list)
    bpy.utils.unregister_class(ACTION_FX_CONFIGS_OT_new_item)
    bpy.utils.unregister_class(ACTION_FX_CONFIGS_OT_delete_item)


# ====================================================================
# UI PANELS
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


class DOPESHEET_PT_action_fx_panel(bpy.types.Panel):
    bl_label = "Action FX Config (EmitTrail)"
    bl_idname = "DOPESHEET_PT_action_fx_panel"
    bl_space_type = 'DOPESHEET_EDITOR'
    bl_region_type = 'UI'
    bl_category = "Action FX"

    @classmethod
    def poll(cls, context):
        return context.object and context.object.animation_data and context.object.animation_data.action

    def draw(self, context):
        layout = self.layout
        action = context.object.animation_data.action

        row = layout.row()
        row.template_list("MY_UL_action_fx_list", "", action, "ashiojin_fx_configs", action, "ashiojin_fx_configs_index")

        row.operator(ACTION_FX_CONFIGS_OT_new_item.bl_idname, text="Add FX Config", icon='ADD')

# ====================================================================
# REGISTRATION BOILERPLATE
# ====================================================================

classes = (
    PANEL_PT_my_engine_settings,
    PANEL_PT_my_engine_object_settings,
    DOPESHEET_PT_action_fx_panel,
)

fn_register, fn_unregister = bpy.utils.register_classes_factory(classes)

def register():
    register_properties()
    fn_register()

def unregister():
    fn_unregister()
    unregister_properties()


# ====================================================================
# GLTF EXPORTER EXTENSION HOOKS
# ====================================================================

class glTF2ExportUserExtension:
    def __init__(self):
        from io_scene_gltf2.io.com.gltf2_io_extensions import Extension
        self.Extension = Extension

    # --- Material Hook ---
    def gather_material_hook(self, gltf2_material, blender_material, export_settings):
        print(f"Processing Material: {blender_material.name}")
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
        print(f"Processing Mesh: {blender_mesh.name}")

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

    def animation_action_hook(self, gltf2_animation, blender_object, action_data, export_settings):
        """
        Triggers for every Action being exported into the glTF file.
        """
        print(f"Processing Action: {blender_object.data.name} {action_data.name}")
        if not hasattr(action_data.action, "ashiojin_fx_configs"):
            print(f"not ashiojin_fx_configs")
            return

        scene = bpy.context.scene
        fps = scene.render.fps / scene.render.fps_base
        fx_config_list = []
        for fx_config in action_data.action.ashiojin_fx_configs:
            if fx_config.target_mesh:
                #print(f"name?{fx_config.target_mesh.data.name}")
                fx_config_list.append({
                    "target_name": fx_config.target_mesh.data.name,
                    "start_sec": fx_config.start_frame / fps,
                    "end_sec": fx_config.end_frame / fps
                })
            else:
                print(f"Warning: FX config in action '{action_data.action.name}' has no valid target mesh.")

        
        # Only inject if a valid target tracking mesh has been picked
        if fx_config_list:
            print(f"Injecting FX config into glTF animation: {fx_config.target_mesh.name}")
            if gltf2_animation.extensions is None:
                gltf2_animation.extensions = {}
            
            # Create a tailored data structure mapping to your exact properties
            extension_name = "ASHIOJIN_action_fx_config"
            gltf2_animation.extensions[extension_name] = self.Extension(
                name=extension_name,
                extension={
                    "fx_configs": fx_config_list
                },
                required=False
            )

if __name__ == "__main__":
    register()
